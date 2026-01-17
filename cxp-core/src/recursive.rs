//! Recursive CXP Support
//!
//! Enables CXP files to contain references to other CXP files,
//! creating a hierarchical tree structure for organizing large datasets.
//!
//! # Structure
//! ```text
//! master.cxp
//! ├── manifest.msgpack (includes children references)
//! ├── children/
//! │   ├── projects.cxpref
//! │   ├── documents.cxpref
//! │   └── media.cxpref
//! └── ...
//! ```

use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

use crate::Result;

/// Reference to a child CXP file
///
/// This is stored in the parent CXP's `children/` directory
/// and contains metadata for lazy loading decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CxpRef {
    /// Unique identifier for this reference
    pub id: String,

    /// Human-readable name (e.g., "Desktop", "Projects", "2024 Photos")
    pub name: String,

    /// How this CXP is stored
    pub storage: CxpStorage,

    /// Metadata for quick access without loading the CXP
    pub meta: CxpRefMeta,

    /// When this CXP was last accessed
    pub last_accessed: Option<DateTime<Utc>>,

    /// Computed tier based on age and access patterns
    #[serde(default)]
    pub tier: FileTier,

    /// Tags for categorization and search
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CxpRef {
    /// Create a new CXP reference
    pub fn new(id: impl Into<String>, name: impl Into<String>, storage: CxpStorage) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            storage,
            meta: CxpRefMeta::default(),
            last_accessed: None,
            tier: FileTier::Warm,
            tags: Vec::new(),
        }
    }

    /// Create reference for an external CXP file
    pub fn external(id: impl Into<String>, name: impl Into<String>, path: PathBuf) -> Self {
        Self::new(id, name, CxpStorage::External { path })
    }

    /// Create reference for an embedded CXP (inside parent ZIP)
    pub fn embedded(id: impl Into<String>, name: impl Into<String>, path_in_zip: impl Into<String>) -> Self {
        Self::new(id, name, CxpStorage::Embedded { path_in_zip: path_in_zip.into() })
    }

    /// Update the tier based on current time
    pub fn recalculate_tier(&mut self) {
        self.tier = self.calculate_tier();
    }

    /// Calculate tier based on modification and access times
    pub fn calculate_tier(&self) -> FileTier {
        let now = Utc::now();

        // Days since last modification
        let days_since_modified = (now - self.meta.updated_at).num_days();

        // Days since last access (default to 365 if never accessed)
        let days_since_accessed = self.last_accessed
            .map(|t| (now - t).num_days())
            .unwrap_or(365);

        // Weighted score: modification is more important than access
        let score = (days_since_modified as f64 * 0.7) + (days_since_accessed as f64 * 0.3);

        match score as i64 {
            0..=7 => FileTier::Hot,     // Active in last week
            8..=30 => FileTier::Warm,   // Active in last month
            _ => FileTier::Cold,        // Archived
        }
    }

    /// Mark this CXP as accessed now
    pub fn touch(&mut self) {
        self.last_accessed = Some(Utc::now());
        self.recalculate_tier();
    }

    /// Check if this is an external CXP file
    pub fn is_external(&self) -> bool {
        matches!(self.storage, CxpStorage::External { .. })
    }

    /// Check if this is embedded in the parent CXP
    pub fn is_embedded(&self) -> bool {
        matches!(self.storage, CxpStorage::Embedded { .. })
    }

    /// Get the path for external CXPs
    pub fn external_path(&self) -> Option<&PathBuf> {
        match &self.storage {
            CxpStorage::External { path } => Some(path),
            _ => None,
        }
    }

    /// Serialize to MessagePack for storage
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self)
            .map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }

    /// Deserialize from MessagePack
    pub fn from_msgpack(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data)
            .map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }
}

/// How a child CXP is stored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CxpStorage {
    /// CXP is embedded within the parent CXP file
    /// Good for small, frequently accessed children
    Embedded {
        /// Path within the ZIP archive (e.g., "children/config.cxp")
        path_in_zip: String,
    },

    /// CXP is a separate file on disk
    /// Good for large children that need independent updates
    External {
        /// Absolute or relative path to the CXP file
        path: PathBuf,
    },

    /// CXP is stored on a remote server (future feature)
    Remote {
        /// URL to the CXP file
        url: String,
        /// SHA-256 checksum for verification
        checksum: String,
    },
}

impl CxpStorage {
    /// Create external storage with path resolution
    pub fn external_at(base_dir: &PathBuf, name: &str) -> Self {
        let path = base_dir.join(format!("{}.cxp", name));
        CxpStorage::External { path }
    }
}

/// Metadata about a child CXP for quick access decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CxpRefMeta {
    /// Optional description
    pub description: Option<String>,

    /// Total number of files in this CXP (including all nested children)
    pub total_files: usize,

    /// Total number of direct children CXPs
    pub child_count: usize,

    /// Does this CXP have children of its own?
    pub has_children: bool,

    /// Compressed size in bytes
    pub size_bytes: u64,

    /// Original (uncompressed) size in bytes
    pub original_size_bytes: u64,

    /// When this CXP was created
    pub created_at: DateTime<Utc>,

    /// When this CXP was last modified
    pub updated_at: DateTime<Utc>,

    /// Category for organization (e.g., "projects", "documents", "media")
    pub category: Option<String>,

    /// File types contained (e.g., ["rs", "ts", "md"])
    #[serde(default)]
    pub file_types: Vec<String>,

    /// Top keywords extracted from content (for global search)
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Has embeddings been generated?
    #[serde(default)]
    pub has_embeddings: bool,
}

impl Default for CxpRefMeta {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            description: None,
            total_files: 0,
            child_count: 0,
            has_children: false,
            size_bytes: 0,
            original_size_bytes: 0,
            created_at: now,
            updated_at: now,
            category: None,
            file_types: Vec::new(),
            keywords: Vec::new(),
            has_embeddings: false,
        }
    }
}

impl CxpRefMeta {
    /// Create metadata from a manifest
    pub fn from_manifest(manifest: &crate::Manifest) -> Self {
        Self {
            description: None,
            total_files: manifest.stats.total_files,
            child_count: 0, // Will be set separately
            has_children: false,
            size_bytes: manifest.stats.cxp_size_bytes,
            original_size_bytes: manifest.stats.original_size_bytes,
            created_at: manifest.created_at,
            updated_at: manifest.updated_at,
            category: None,
            file_types: manifest.file_types.keys().cloned().collect(),
            keywords: manifest.topics.clone(),
            has_embeddings: manifest.embedding_model.is_some(),
        }
    }

    /// Update from a manifest (preserves category and keywords)
    pub fn update_from_manifest(&mut self, manifest: &crate::Manifest) {
        self.total_files = manifest.stats.total_files;
        self.size_bytes = manifest.stats.cxp_size_bytes;
        self.original_size_bytes = manifest.stats.original_size_bytes;
        self.updated_at = manifest.updated_at;
        self.file_types = manifest.file_types.keys().cloned().collect();
        self.has_embeddings = manifest.embedding_model.is_some();
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size_bytes == 0 {
            1.0
        } else {
            self.size_bytes as f64 / self.original_size_bytes as f64
        }
    }

    /// Human-readable size
    pub fn size_display(&self) -> String {
        let bytes = self.size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
        } else {
            format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
        }
    }
}

/// File/CXP tier for priority-based loading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum FileTier {
    /// Hot: Always keep in memory, frequently accessed
    /// - Active projects
    /// - Desktop files
    /// - Recent downloads
    Hot = 0,

    /// Warm: Load on demand, recently used
    /// - Documents from this month
    /// - Recent projects
    #[default]
    Warm = 1,

    /// Cold: Only load when explicitly requested
    /// - Old archives
    /// - Backup data
    /// - Historical records
    Cold = 2,
}

impl FileTier {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            FileTier::Hot => "Hot",
            FileTier::Warm => "Warm",
            FileTier::Cold => "Cold",
        }
    }

    /// Get emoji for display
    pub fn emoji(&self) -> &'static str {
        match self {
            FileTier::Hot => "🔥",
            FileTier::Warm => "🟡",
            FileTier::Cold => "🧊",
        }
    }

    /// Should this tier be preloaded on startup?
    pub fn should_preload(&self) -> bool {
        matches!(self, FileTier::Hot)
    }

    /// Memory priority (lower = higher priority to keep)
    pub fn eviction_priority(&self) -> u8 {
        match self {
            FileTier::Hot => 0,   // Never evict
            FileTier::Warm => 1,  // Evict if needed
            FileTier::Cold => 2,  // Evict first
        }
    }
}

/// Collection of child references in a CXP
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChildrenMap {
    /// Map of child ID to reference
    pub children: std::collections::HashMap<String, CxpRef>,

    /// Ordered list of child IDs (for consistent iteration)
    pub order: Vec<String>,
}

impl ChildrenMap {
    /// Create an empty children map
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a child reference
    pub fn add(&mut self, child: CxpRef) {
        let id = child.id.clone();
        if !self.children.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.children.insert(id, child);
    }

    /// Remove a child by ID
    pub fn remove(&mut self, id: &str) -> Option<CxpRef> {
        self.order.retain(|i| i != id);
        self.children.remove(id)
    }

    /// Get a child by ID
    pub fn get(&self, id: &str) -> Option<&CxpRef> {
        self.children.get(id)
    }

    /// Get a mutable reference to a child
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CxpRef> {
        self.children.get_mut(id)
    }

    /// Iterate over children in order
    pub fn iter(&self) -> impl Iterator<Item = &CxpRef> {
        self.order.iter().filter_map(|id| self.children.get(id))
    }

    /// Number of children
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Get children by tier
    pub fn by_tier(&self, tier: FileTier) -> Vec<&CxpRef> {
        self.iter().filter(|c| c.tier == tier).collect()
    }

    /// Get all hot children (for preloading)
    pub fn hot(&self) -> Vec<&CxpRef> {
        self.by_tier(FileTier::Hot)
    }

    /// Serialize to MessagePack
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self)
            .map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }

    /// Deserialize from MessagePack
    pub fn from_msgpack(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data)
            .map_err(|e| crate::CxpError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cxp_ref_creation() {
        let child = CxpRef::external("proj-1", "My Project", PathBuf::from("/path/to/project.cxp"));

        assert_eq!(child.id, "proj-1");
        assert_eq!(child.name, "My Project");
        assert!(child.is_external());
        assert!(!child.is_embedded());
    }

    #[test]
    fn test_tier_calculation() {
        let mut child = CxpRef::external("test", "Test", PathBuf::from("/test.cxp"));

        // Just created and accessed = Hot
        child.meta.updated_at = Utc::now();
        child.last_accessed = Some(Utc::now());
        assert_eq!(child.calculate_tier(), FileTier::Hot);

        // 10 days old and accessed = Warm
        child.meta.updated_at = Utc::now() - chrono::Duration::days(10);
        child.last_accessed = Some(Utc::now() - chrono::Duration::days(10));
        assert_eq!(child.calculate_tier(), FileTier::Warm);

        // 60 days old = Cold
        child.meta.updated_at = Utc::now() - chrono::Duration::days(60);
        child.last_accessed = Some(Utc::now() - chrono::Duration::days(60));
        assert_eq!(child.calculate_tier(), FileTier::Cold);
    }

    #[test]
    fn test_children_map() {
        let mut children = ChildrenMap::new();

        let child1 = CxpRef::external("a", "First", PathBuf::from("/a.cxp"));
        let child2 = CxpRef::external("b", "Second", PathBuf::from("/b.cxp"));

        children.add(child1);
        children.add(child2);

        assert_eq!(children.len(), 2);
        assert_eq!(children.get("a").unwrap().name, "First");
        assert_eq!(children.get("b").unwrap().name, "Second");

        // Order is preserved
        let names: Vec<_> = children.iter().map(|c| &c.name).collect();
        assert_eq!(names, vec!["First", "Second"]);
    }

    #[test]
    fn test_serialization() {
        let child = CxpRef::external("test", "Test CXP", PathBuf::from("/test.cxp"));

        let bytes = child.to_msgpack().unwrap();
        let restored = CxpRef::from_msgpack(&bytes).unwrap();

        assert_eq!(restored.id, child.id);
        assert_eq!(restored.name, child.name);
    }
}
