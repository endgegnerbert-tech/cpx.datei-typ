//! CXP Manifest - The "roadmap" for AI to understand the file
//!
//! Contains metadata, statistics, and structure information.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::recursive::{ChildrenMap, FileTier};

/// CXP Manifest - stored as manifest.msgpack in the CXP file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// CXP format version
    pub version: String,

    /// When the CXP file was created
    pub created_at: DateTime<Utc>,

    /// When the CXP file was last updated
    pub updated_at: DateTime<Utc>,

    /// Statistics about the contents
    pub stats: ManifestStats,

    /// File type breakdown
    pub file_types: HashMap<String, FileTypeInfo>,

    /// Top topics/categories detected
    pub topics: Vec<String>,

    /// Embedding model used (if embeddings are present)
    #[serde(default)]
    pub embedding_model: Option<String>,

    /// Embedding dimension
    #[serde(default)]
    pub embedding_dim: Option<usize>,

    /// Extensions present in this CXP file
    pub extensions: Vec<String>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,

    // === Recursive CXP Support ===

    /// Child CXP references (for hierarchical structures)
    #[serde(default)]
    pub children: ChildrenMap,

    /// Parent CXP path (if this is a child CXP)
    #[serde(default)]
    pub parent_path: Option<Vec<String>>,

    /// Tier of this CXP (Hot/Warm/Cold)
    #[serde(default)]
    pub tier: FileTier,

    /// Categories for this CXP
    #[serde(default)]
    pub categories: Vec<String>,

    /// Keywords extracted from content (for global search)
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Last access time (for tier calculation)
    #[serde(default)]
    pub last_accessed: Option<DateTime<Utc>>,
}

/// Statistics about the CXP contents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestStats {
    /// Total number of source files
    pub total_files: usize,

    /// Total number of unique chunks
    pub unique_chunks: usize,

    /// Original size in bytes (before chunking/compression)
    pub original_size_bytes: u64,

    /// CXP file size in bytes
    pub cxp_size_bytes: u64,

    /// Compression ratio (cxp_size / original_size)
    pub compression_ratio: f64,

    /// Deduplication savings percentage
    pub dedup_savings_percent: f64,
}

/// Information about a file type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeInfo {
    /// Number of files with this extension
    pub count: usize,

    /// Human-readable description
    pub description: String,

    /// Example file paths
    pub sample_files: Vec<String>,

    /// Total bytes for this file type
    pub total_bytes: u64,
}

impl Manifest {
    /// Create a new manifest with default values
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            version: crate::VERSION.to_string(),
            created_at: now,
            updated_at: now,
            stats: ManifestStats::default(),
            file_types: HashMap::new(),
            topics: Vec::new(),
            embedding_model: None,
            embedding_dim: None,
            extensions: Vec::new(),
            metadata: HashMap::new(),
            // Recursive CXP defaults
            children: ChildrenMap::new(),
            parent_path: None,
            tier: FileTier::Warm,
            categories: Vec::new(),
            keywords: Vec::new(),
            last_accessed: None,
        }
    }

    /// Check if this CXP has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get child count
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Add a child reference
    pub fn add_child(&mut self, child: crate::recursive::CxpRef) {
        self.children.add(child);
    }

    /// Get children by tier
    pub fn hot_children(&self) -> Vec<&crate::recursive::CxpRef> {
        self.children.hot()
    }

    /// Calculate the current tier based on access patterns
    pub fn calculate_tier(&self) -> FileTier {
        let now = Utc::now();
        let days_since_modified = (now - self.updated_at).num_days();
        let days_since_accessed = self.last_accessed
            .map(|t| (now - t).num_days())
            .unwrap_or(365);

        // Weighted score: modification is more important than access
        let score = (days_since_modified as f64 * 0.7) + (days_since_accessed as f64 * 0.3);

        match score as i64 {
            0..=7 => FileTier::Hot,
            8..=30 => FileTier::Warm,
            _ => FileTier::Cold,
        }
    }

    /// Update tier based on current access patterns
    pub fn recalculate_tier(&mut self) {
        self.tier = self.calculate_tier();
    }

    /// Mark as accessed
    pub fn touch_access(&mut self) {
        self.last_accessed = Some(Utc::now());
        self.recalculate_tier();
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Add a file type entry
    pub fn add_file_type(&mut self, ext: &str, path: &str, size: u64) {
        let entry = self.file_types.entry(ext.to_lowercase()).or_insert_with(|| {
            FileTypeInfo {
                count: 0,
                description: get_file_type_description(ext),
                sample_files: Vec::new(),
                total_bytes: 0,
            }
        });

        entry.count += 1;
        entry.total_bytes += size;

        // Keep up to 3 sample files
        if entry.sample_files.len() < 3 {
            entry.sample_files.push(path.to_string());
        }
    }

    /// Serialize to MessagePack
    pub fn to_msgpack(&self) -> crate::Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }

    /// Deserialize from MessagePack
    pub fn from_msgpack(data: &[u8]) -> crate::Result<Self> {
        rmp_serde::from_slice(data).map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }

    /// Serialize to JSON (for debugging/human reading)
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Get human-readable description for a file extension
fn get_file_type_description(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        // Programming Languages
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "py" => "Python",
        "go" => "Go",
        "java" => "Java",
        "c" => "C",
        "cpp" | "cc" | "cxx" => "C++",
        "h" | "hpp" => "C/C++ Header",
        "cs" => "C#",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" => "Kotlin",
        "scala" => "Scala",
        "r" => "R",
        "sql" => "SQL",

        // Shell
        "sh" | "bash" => "Bash Script",
        "zsh" => "Zsh Script",
        "ps1" => "PowerShell",
        "bat" | "cmd" => "Windows Batch",

        // Config
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "ini" => "INI Config",
        "env" => "Environment Variables",

        // Documentation
        "md" | "mdx" => "Markdown",
        "txt" => "Plain Text",
        "rst" => "reStructuredText",
        "adoc" => "AsciiDoc",
        "tex" => "LaTeX",

        // Web
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS/Sass",
        "less" => "Less",
        "vue" => "Vue Component",
        "svelte" => "Svelte Component",

        // Data
        "csv" => "CSV",
        "tsv" => "TSV",

        _ => "Unknown",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new();
        assert_eq!(manifest.version, crate::VERSION);
        assert!(manifest.file_types.is_empty());
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = Manifest::new();
        manifest.add_file_type("rs", "src/main.rs", 1000);
        manifest.add_file_type("rs", "src/lib.rs", 500);
        manifest.add_file_type("ts", "src/app.ts", 2000);

        // Serialize to msgpack
        let data = manifest.to_msgpack().unwrap();
        assert!(!data.is_empty());

        // Deserialize back
        let restored = Manifest::from_msgpack(&data).unwrap();
        assert_eq!(restored.version, manifest.version);
        assert_eq!(restored.file_types.len(), 2);
        assert_eq!(restored.file_types.get("rs").unwrap().count, 2);
    }
}
