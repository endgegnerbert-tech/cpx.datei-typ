//! Global Search Index for Recursive CXPs
//!
//! Provides fast cross-CXP search without loading all CXPs into memory.
//! Uses a lightweight index with keywords and optional embedding hashes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::recursive::FileTier;
use crate::Result;

/// Global index spanning all CXPs in the hierarchy
///
/// This index is stored in the master CXP and provides
/// fast lookup across all children without loading them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalIndex {
    /// All indexed entries
    pub entries: Vec<GlobalIndexEntry>,

    /// CXP path -> index range for fast filtering
    pub cxp_ranges: HashMap<String, IndexRange>,

    /// Keyword -> entry indices for keyword search
    #[serde(skip)]
    keyword_index: HashMap<String, Vec<usize>>,

    /// File type -> entry indices
    #[serde(skip)]
    type_index: HashMap<String, Vec<usize>>,

    /// Statistics
    pub stats: GlobalIndexStats,
}

/// Range of entries belonging to a specific CXP
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexRange {
    pub start: usize,
    pub end: usize,
}

/// Statistics about the global index
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalIndexStats {
    /// Total number of entries
    pub total_entries: usize,

    /// Number of CXPs indexed
    pub cxp_count: usize,

    /// Total files across all CXPs
    pub total_files: usize,

    /// When the index was last updated
    pub updated_at: Option<DateTime<Utc>>,

    /// Index size in bytes (approximate)
    pub index_size_bytes: usize,
}

/// A single entry in the global index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalIndexEntry {
    /// Which CXP contains this file
    pub cxp_id: String,

    /// Full path to the CXP (e.g., ["home", "projects", "contextai"])
    pub cxp_path: Vec<String>,

    /// File path within the CXP
    pub file_path: String,

    /// File name (for quick display)
    pub file_name: String,

    /// File type/extension
    pub file_type: String,

    /// File size in bytes
    pub file_size: u64,

    /// Keywords extracted from content (max 20)
    pub keywords: Vec<String>,

    /// Tier of the containing CXP
    pub tier: FileTier,

    /// When the file was last modified
    pub modified_at: DateTime<Utc>,

    /// Optional: Hash of the embedding vector for similarity search
    /// Uses SimHash for approximate matching
    #[serde(default)]
    pub embedding_hash: Option<u64>,

    /// Optional: First 200 chars of content for preview
    #[serde(default)]
    pub preview: Option<String>,
}

impl GlobalIndexEntry {
    /// Create a new index entry
    pub fn new(
        cxp_id: impl Into<String>,
        cxp_path: Vec<String>,
        file_path: impl Into<String>,
        file_type: impl Into<String>,
    ) -> Self {
        let file_path = file_path.into();
        let file_name = file_path
            .rsplit('/')
            .next()
            .unwrap_or(&file_path)
            .to_string();

        Self {
            cxp_id: cxp_id.into(),
            cxp_path,
            file_path,
            file_name,
            file_type: file_type.into(),
            file_size: 0,
            keywords: Vec::new(),
            tier: FileTier::Warm,
            modified_at: Utc::now(),
            embedding_hash: None,
            preview: None,
        }
    }

    /// Match against a search query
    pub fn matches(&self, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut score = 0.0;

        // Exact file name match (highest score)
        if self.file_name.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }

        // File path match
        if self.file_path.to_lowercase().contains(&query_lower) {
            score += 5.0;
        }

        // Keyword matches
        for term in &query_terms {
            for keyword in &self.keywords {
                if keyword.to_lowercase().contains(term) {
                    score += 2.0;
                } else if term.len() > 3 && keyword.to_lowercase().starts_with(term) {
                    score += 1.0;
                }
            }
        }

        // File type match
        if self.file_type.to_lowercase() == query_lower {
            score += 3.0;
        }

        // Preview match
        if let Some(ref preview) = self.preview {
            for term in &query_terms {
                if preview.to_lowercase().contains(term) {
                    score += 1.0;
                }
            }
        }

        // Tier bonus (prefer hot content)
        match self.tier {
            FileTier::Hot => score *= 1.2,
            FileTier::Warm => score *= 1.0,
            FileTier::Cold => score *= 0.8,
        }

        score
    }

    /// Get the full display path
    pub fn display_path(&self) -> String {
        if self.cxp_path.is_empty() {
            self.file_path.clone()
        } else {
            format!("{}/{}", self.cxp_path.join("/"), self.file_path)
        }
    }
}

impl GlobalIndex {
    /// Create a new empty global index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entry to the index
    pub fn add(&mut self, entry: GlobalIndexEntry) {
        let index = self.entries.len();

        // Update keyword index
        for keyword in &entry.keywords {
            let keyword_lower = keyword.to_lowercase();
            self.keyword_index
                .entry(keyword_lower)
                .or_default()
                .push(index);
        }

        // Update type index
        let type_lower = entry.file_type.to_lowercase();
        self.type_index
            .entry(type_lower)
            .or_default()
            .push(index);

        // Update CXP ranges
        let cxp_key = entry.cxp_path.join("/");
        if let Some(range) = self.cxp_ranges.get_mut(&cxp_key) {
            range.end = index + 1;
        } else {
            self.cxp_ranges.insert(cxp_key, IndexRange {
                start: index,
                end: index + 1,
            });
        }

        self.entries.push(entry);
        self.stats.total_entries = self.entries.len();
    }

    /// Add multiple entries from a CXP
    pub fn add_from_cxp(&mut self, cxp_id: &str, cxp_path: Vec<String>, entries: Vec<GlobalIndexEntry>) {
        for mut entry in entries {
            entry.cxp_id = cxp_id.to_string();
            entry.cxp_path = cxp_path.clone();
            self.add(entry);
        }
        self.stats.cxp_count = self.cxp_ranges.len();
    }

    /// Remove all entries for a specific CXP
    pub fn remove_cxp(&mut self, cxp_path: &[String]) {
        let key = cxp_path.join("/");
        if let Some(range) = self.cxp_ranges.remove(&key) {
            // Mark entries as removed (we don't actually remove to preserve indices)
            // In a real implementation, we'd compact the index periodically
            for i in range.start..range.end {
                if i < self.entries.len() {
                    self.entries[i].cxp_id = String::new(); // Mark as removed
                }
            }
        }
    }

    /// Search the index
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let mut results: Vec<_> = self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.cxp_id.is_empty()) // Skip removed entries
            .map(|(idx, entry)| {
                let score = entry.matches(query);
                SearchResult {
                    index: idx,
                    entry: entry.clone(),
                    score,
                }
            })
            .filter(|r| r.score > 0.0)
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        results.truncate(limit);
        results
    }

    /// Search by file type
    pub fn search_by_type(&self, file_type: &str, limit: usize) -> Vec<&GlobalIndexEntry> {
        let type_lower = file_type.to_lowercase();

        if let Some(indices) = self.type_index.get(&type_lower) {
            indices
                .iter()
                .take(limit)
                .filter_map(|&idx| self.entries.get(idx))
                .filter(|e| !e.cxp_id.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Search by keyword
    pub fn search_by_keyword(&self, keyword: &str, limit: usize) -> Vec<&GlobalIndexEntry> {
        let keyword_lower = keyword.to_lowercase();

        if let Some(indices) = self.keyword_index.get(&keyword_lower) {
            indices
                .iter()
                .take(limit)
                .filter_map(|&idx| self.entries.get(idx))
                .filter(|e| !e.cxp_id.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get entries for a specific CXP
    pub fn entries_for_cxp(&self, cxp_path: &[String]) -> Vec<&GlobalIndexEntry> {
        let key = cxp_path.join("/");

        if let Some(range) = self.cxp_ranges.get(&key) {
            self.entries[range.start..range.end]
                .iter()
                .filter(|e| !e.cxp_id.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get entries by tier
    pub fn entries_by_tier(&self, tier: FileTier) -> Vec<&GlobalIndexEntry> {
        self.entries
            .iter()
            .filter(|e| !e.cxp_id.is_empty() && e.tier == tier)
            .collect()
    }

    /// Rebuild the in-memory indices after deserialization
    pub fn rebuild_indices(&mut self) {
        self.keyword_index.clear();
        self.type_index.clear();

        for (index, entry) in self.entries.iter().enumerate() {
            if entry.cxp_id.is_empty() {
                continue; // Skip removed entries
            }

            // Rebuild keyword index
            for keyword in &entry.keywords {
                let keyword_lower = keyword.to_lowercase();
                self.keyword_index
                    .entry(keyword_lower)
                    .or_default()
                    .push(index);
            }

            // Rebuild type index
            let type_lower = entry.file_type.to_lowercase();
            self.type_index
                .entry(type_lower)
                .or_default()
                .push(index);
        }
    }

    /// Compact the index by removing empty entries
    pub fn compact(&mut self) {
        // Remove entries with empty cxp_id
        self.entries.retain(|e| !e.cxp_id.is_empty());

        // Rebuild all indices
        self.cxp_ranges.clear();
        self.rebuild_indices();

        // Rebuild CXP ranges
        for (index, entry) in self.entries.iter().enumerate() {
            let key = entry.cxp_path.join("/");
            if let Some(range) = self.cxp_ranges.get_mut(&key) {
                range.end = index + 1;
            } else {
                self.cxp_ranges.insert(key, IndexRange {
                    start: index,
                    end: index + 1,
                });
            }
        }

        // Update stats
        self.stats.total_entries = self.entries.len();
        self.stats.cxp_count = self.cxp_ranges.len();
        self.stats.updated_at = Some(Utc::now());
    }

    /// Serialize to MessagePack
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self)
            .map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }

    /// Deserialize from MessagePack
    pub fn from_msgpack(data: &[u8]) -> Result<Self> {
        let mut index: Self = rmp_serde::from_slice(data)
            .map_err(|e| crate::CxpError::Serialization(e.to_string()))?;
        index.rebuild_indices();
        Ok(index)
    }

    /// Estimate memory size
    pub fn memory_size(&self) -> usize {
        let entries_size = self.entries.len() * std::mem::size_of::<GlobalIndexEntry>();
        let keyword_size = self.keyword_index.len() * 64; // Approximate
        let type_size = self.type_index.len() * 32;

        entries_size + keyword_size + type_size
    }
}

/// Search result with score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Index in the entries array
    pub index: usize,
    /// The matching entry
    pub entry: GlobalIndexEntry,
    /// Relevance score
    pub score: f32,
}

impl SearchResult {
    /// Get the CXP path to load for deep search
    pub fn cxp_path(&self) -> &[String] {
        &self.entry.cxp_path
    }

    /// Get a short display string
    pub fn display(&self) -> String {
        format!(
            "{} {} ({})",
            self.entry.tier.emoji(),
            self.entry.file_name,
            self.entry.cxp_path.last().unwrap_or(&"root".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_search() {
        let mut index = GlobalIndex::new();

        let mut entry1 = GlobalIndexEntry::new("cxp1", vec!["projects".to_string()], "src/main.rs", "rs");
        entry1.keywords = vec!["rust".to_string(), "main".to_string(), "entry".to_string()];

        let mut entry2 = GlobalIndexEntry::new("cxp1", vec!["projects".to_string()], "src/lib.rs", "rs");
        entry2.keywords = vec!["rust".to_string(), "library".to_string()];

        index.add(entry1);
        index.add(entry2);

        let results = index.search("rust main", 10);
        assert!(!results.is_empty());
        assert!(results[0].entry.file_name.contains("main"));
    }

    #[test]
    fn test_search_by_type() {
        let mut index = GlobalIndex::new();

        index.add(GlobalIndexEntry::new("c1", vec![], "file.rs", "rs"));
        index.add(GlobalIndexEntry::new("c1", vec![], "file.ts", "ts"));
        index.add(GlobalIndexEntry::new("c1", vec![], "file2.rs", "rs"));

        let rs_files = index.search_by_type("rs", 10);
        assert_eq!(rs_files.len(), 2);

        let ts_files = index.search_by_type("ts", 10);
        assert_eq!(ts_files.len(), 1);
    }

    #[test]
    fn test_tier_scoring() {
        let mut hot = GlobalIndexEntry::new("c1", vec![], "hot.rs", "rs");
        hot.tier = FileTier::Hot;
        hot.keywords = vec!["test".to_string()];

        let mut cold = GlobalIndexEntry::new("c1", vec![], "cold.rs", "rs");
        cold.tier = FileTier::Cold;
        cold.keywords = vec!["test".to_string()];

        // Hot should score higher
        assert!(hot.matches("test") > cold.matches("test"));
    }

    #[test]
    fn test_serialization() {
        let mut index = GlobalIndex::new();

        let mut entry = GlobalIndexEntry::new("cxp1", vec!["home".to_string()], "test.txt", "txt");
        entry.keywords = vec!["hello".to_string(), "world".to_string()];
        index.add(entry);

        let bytes = index.to_msgpack().unwrap();
        let restored = GlobalIndex::from_msgpack(&bytes).unwrap();

        assert_eq!(restored.entries.len(), 1);
        assert_eq!(restored.entries[0].file_name, "test.txt");
    }
}
