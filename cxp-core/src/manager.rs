//! CXP Manager with LRU Cache
//!
//! Manages recursive CXP hierarchies with lazy loading and memory management.
//! Hot CXPs stay in memory, Warm/Cold are loaded on demand.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use chrono::Utc;

use crate::recursive::{CxpRef, CxpStorage, ChildrenMap, FileTier};
use crate::global_index::{GlobalIndex, GlobalIndexEntry};
use crate::format::CxpFile;
use crate::Result;
use crate::CxpError;

/// Configuration for the CXP Manager
#[derive(Debug, Clone)]
pub struct CxpManagerConfig {
    /// Maximum memory usage in bytes (default: 500 MB)
    pub max_memory_bytes: usize,

    /// Maximum number of CXPs to keep in cache
    pub max_cached_cxps: usize,

    /// Root directory for CXP storage
    pub storage_root: PathBuf,

    /// Preload all Hot CXPs on startup
    pub preload_hot: bool,
}

impl Default for CxpManagerConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 500 * 1024 * 1024, // 500 MB
            max_cached_cxps: 50,
            storage_root: PathBuf::from("~/.contextai/"),
            preload_hot: true,
        }
    }
}

/// LRU Entry for tracking cached CXPs
#[derive(Debug)]
struct CacheEntry {
    /// The loaded CXP file
    cxp: CxpFile,

    /// Last access time for LRU
    last_accessed: chrono::DateTime<Utc>,

    /// Approximate memory size
    memory_size: usize,

    /// Tier of this CXP
    tier: FileTier,
}

/// Manager for recursive CXP hierarchies
///
/// Provides:
/// - Lazy loading of CXPs based on tier
/// - LRU cache with memory limits
/// - Cross-CXP search via GlobalIndex
/// - Automatic tier recalculation
pub struct CxpManager {
    /// Configuration
    config: CxpManagerConfig,

    /// Global index for fast search
    global_index: Arc<RwLock<GlobalIndex>>,

    /// Loaded CXPs (LRU cache)
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,

    /// Root CXP references (children of master.cxp)
    root_children: Arc<RwLock<ChildrenMap>>,

    /// Current memory usage
    current_memory: Arc<RwLock<usize>>,

    /// LRU order (most recent at end)
    lru_order: Arc<RwLock<Vec<String>>>,
}

impl CxpManager {
    /// Create a new CXP Manager
    pub fn new(config: CxpManagerConfig) -> Self {
        Self {
            config,
            global_index: Arc::new(RwLock::new(GlobalIndex::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            root_children: Arc::new(RwLock::new(ChildrenMap::new())),
            current_memory: Arc::new(RwLock::new(0)),
            lru_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize the manager, loading root CXP references
    pub fn init(&self) -> Result<()> {
        let master_path = self.config.storage_root.join("master.cxp");

        if master_path.exists() {
            // Load master CXP to get children references
            self.load_master_refs(&master_path)?;

            if self.config.preload_hot {
                self.preload_hot_cxps()?;
            }
        }

        Ok(())
    }

    /// Load master CXP references
    fn load_master_refs(&self, master_path: &Path) -> Result<()> {
        // Read the master CXP's children directory
        let children_dir = master_path.with_extension("").join("children");

        if !children_dir.exists() {
            return Ok(());
        }

        let mut children = self.root_children.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        for entry in std::fs::read_dir(&children_dir)
            .map_err(|e| CxpError::Io(e.to_string()))?
        {
            let entry = entry.map_err(|e| CxpError::Io(e.to_string()))?;
            let path = entry.path();

            if path.extension().map(|e| e == "cxpref").unwrap_or(false) {
                let data = std::fs::read(&path)
                    .map_err(|e| CxpError::Io(e.to_string()))?;
                let cxp_ref = CxpRef::from_msgpack(&data)?;
                children.add(cxp_ref);
            }
        }

        Ok(())
    }

    /// Preload all Hot tier CXPs into memory
    fn preload_hot_cxps(&self) -> Result<()> {
        let children = self.root_children.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        let hot_refs: Vec<_> = children.hot().iter()
            .map(|r| (*r).clone())
            .collect();

        drop(children); // Release lock before loading

        for cxp_ref in hot_refs {
            let _ = self.load_cxp(&cxp_ref.id); // Ignore errors during preload
        }

        Ok(())
    }

    /// Get a CXP by path (e.g., ["home", "projects", "contextai"])
    pub fn get(&self, path: &[&str]) -> Result<Option<CxpFile>> {
        if path.is_empty() {
            return Ok(None);
        }

        let cxp_id = path.join("/");

        // Check cache first
        if let Some(cxp) = self.get_from_cache(&cxp_id)? {
            return Ok(Some(cxp));
        }

        // Load from disk
        self.load_cxp(&cxp_id)
    }

    /// Get from cache, updating LRU order
    fn get_from_cache(&self, cxp_id: &str) -> Result<Option<CxpFile>> {
        let mut cache = self.cache.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        if let Some(entry) = cache.get_mut(cxp_id) {
            entry.last_accessed = Utc::now();

            // Update LRU order
            let mut lru = self.lru_order.write()
                .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
            lru.retain(|id| id != cxp_id);
            lru.push(cxp_id.to_string());

            return Ok(Some(entry.cxp.clone()));
        }

        Ok(None)
    }

    /// Load a CXP from disk and cache it
    fn load_cxp(&self, cxp_id: &str) -> Result<Option<CxpFile>> {
        // Find the CxpRef for this ID
        let cxp_ref = self.find_ref(cxp_id)?;

        let cxp_ref = match cxp_ref {
            Some(r) => r,
            None => return Ok(None),
        };

        // Get the actual path
        let cxp_path = match &cxp_ref.storage {
            CxpStorage::External { path } => path.clone(),
            CxpStorage::Embedded { path_in_zip } => {
                // TODO: Extract from parent ZIP
                return Err(CxpError::Io(format!(
                    "Embedded CXP loading not yet implemented: {}",
                    path_in_zip
                )));
            }
            CxpStorage::Remote { url, .. } => {
                return Err(CxpError::Io(format!(
                    "Remote CXP loading not yet implemented: {}",
                    url
                )));
            }
        };

        if !cxp_path.exists() {
            return Ok(None);
        }

        // Load the CXP file
        let cxp = CxpFile::open(&cxp_path)?;
        let memory_size = cxp.estimate_memory_size();

        // Evict if necessary
        self.ensure_memory_available(memory_size)?;

        // Add to cache
        let mut cache = self.cache.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        cache.insert(cxp_id.to_string(), CacheEntry {
            cxp: cxp.clone(),
            last_accessed: Utc::now(),
            memory_size,
            tier: cxp_ref.tier,
        });

        // Update LRU order
        let mut lru = self.lru_order.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
        lru.push(cxp_id.to_string());

        // Update memory counter
        let mut memory = self.current_memory.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
        *memory += memory_size;

        Ok(Some(cxp))
    }

    /// Find a CxpRef by ID (recursive search)
    fn find_ref(&self, cxp_id: &str) -> Result<Option<CxpRef>> {
        let children = self.root_children.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        // First check root level
        if let Some(cxp_ref) = children.get(cxp_id) {
            return Ok(Some(cxp_ref.clone()));
        }

        // Check if it's a nested path
        let parts: Vec<&str> = cxp_id.split('/').collect();
        if parts.len() > 1 {
            // TODO: Recursively search in loaded child CXPs
            // For now, just check root level
        }

        Ok(None)
    }

    /// Ensure enough memory is available, evicting if necessary
    fn ensure_memory_available(&self, needed: usize) -> Result<()> {
        let current = *self.current_memory.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        if current + needed <= self.config.max_memory_bytes {
            return Ok(());
        }

        // Need to evict
        let to_evict = (current + needed).saturating_sub(self.config.max_memory_bytes);
        self.evict_lru(to_evict)?;

        Ok(())
    }

    /// Evict least recently used CXPs to free memory
    fn evict_lru(&self, bytes_needed: usize) -> Result<()> {
        let mut freed = 0usize;
        let mut to_remove = Vec::new();

        {
            let cache = self.cache.read()
                .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
            let lru = self.lru_order.read()
                .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

            // Iterate from least recently used (front of list)
            for cxp_id in lru.iter() {
                if freed >= bytes_needed {
                    break;
                }

                if let Some(entry) = cache.get(cxp_id) {
                    // Never evict Hot CXPs
                    if entry.tier == FileTier::Hot {
                        continue;
                    }

                    // Evict Cold first, then Warm
                    to_remove.push((cxp_id.clone(), entry.memory_size));
                    freed += entry.memory_size;
                }
            }
        }

        // Actually remove
        let mut cache = self.cache.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
        let mut lru = self.lru_order.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
        let mut memory = self.current_memory.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        for (cxp_id, size) in to_remove {
            cache.remove(&cxp_id);
            lru.retain(|id| id != &cxp_id);
            *memory = memory.saturating_sub(size);
        }

        Ok(())
    }

    /// Search across all CXPs using the global index
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let index = self.global_index.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        let results = index.search(query, limit);

        Ok(results.into_iter().map(|r| SearchHit {
            cxp_path: r.entry.cxp_path.clone(),
            file_path: r.entry.file_path.clone(),
            file_name: r.entry.file_name.clone(),
            score: r.score,
            preview: r.entry.preview.clone(),
            tier: r.entry.tier,
        }).collect())
    }

    /// Search by file type
    pub fn search_by_type(&self, file_type: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let index = self.global_index.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        let results = index.search_by_type(file_type, limit);

        Ok(results.into_iter().map(|e| SearchHit {
            cxp_path: e.cxp_path.clone(),
            file_path: e.file_path.clone(),
            file_name: e.file_name.clone(),
            score: 1.0,
            preview: e.preview.clone(),
            tier: e.tier,
        }).collect())
    }

    /// Add a CXP to the index
    pub fn index_cxp(&self, cxp_path: &[String], cxp: &CxpFile) -> Result<()> {
        let cxp_id = cxp_path.join("/");

        let entries: Vec<GlobalIndexEntry> = cxp.file_map.files.iter()
            .map(|(path, file_entry)| {
                let mut entry = GlobalIndexEntry::new(
                    &cxp_id,
                    cxp_path.to_vec(),
                    path,
                    &file_entry.extension,
                );
                entry.file_size = file_entry.size;
                // Keywords would be extracted during CXP creation
                entry
            })
            .collect();

        let mut index = self.global_index.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        index.add_from_cxp(&cxp_id, cxp_path.to_vec(), entries);

        Ok(())
    }

    /// Update tier for a CXP reference
    pub fn update_tier(&self, cxp_id: &str) -> Result<()> {
        let mut children = self.root_children.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        if let Some(cxp_ref) = children.get_mut(cxp_id) {
            cxp_ref.recalculate_tier();
        }

        Ok(())
    }

    /// Mark a CXP as accessed (updates tier calculation)
    pub fn touch(&self, cxp_id: &str) -> Result<()> {
        let mut children = self.root_children.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        if let Some(cxp_ref) = children.get_mut(cxp_id) {
            cxp_ref.touch();
        }

        Ok(())
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> Result<MemoryStats> {
        let current = *self.current_memory.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;
        let cache = self.cache.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        let mut hot_count = 0;
        let mut warm_count = 0;
        let mut cold_count = 0;

        for entry in cache.values() {
            match entry.tier {
                FileTier::Hot => hot_count += 1,
                FileTier::Warm => warm_count += 1,
                FileTier::Cold => cold_count += 1,
            }
        }

        Ok(MemoryStats {
            used_bytes: current,
            max_bytes: self.config.max_memory_bytes,
            cached_cxps: cache.len(),
            hot_cxps: hot_count,
            warm_cxps: warm_count,
            cold_cxps: cold_count,
        })
    }

    /// Get all root children
    pub fn root_children(&self) -> Result<Vec<CxpRef>> {
        let children = self.root_children.read()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        Ok(children.iter().cloned().collect())
    }

    /// Add a root child
    pub fn add_root_child(&self, cxp_ref: CxpRef) -> Result<()> {
        let mut children = self.root_children.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        children.add(cxp_ref);
        Ok(())
    }

    /// Remove a root child
    pub fn remove_root_child(&self, cxp_id: &str) -> Result<Option<CxpRef>> {
        let mut children = self.root_children.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        Ok(children.remove(cxp_id))
    }

    /// Compact the global index (remove deleted entries)
    pub fn compact_index(&self) -> Result<()> {
        let mut index = self.global_index.write()
            .map_err(|_| CxpError::Io("Lock poisoned".to_string()))?;

        index.compact();
        Ok(())
    }
}

/// Search result with context
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Path to the containing CXP
    pub cxp_path: Vec<String>,

    /// File path within the CXP
    pub file_path: String,

    /// File name for display
    pub file_name: String,

    /// Relevance score
    pub score: f32,

    /// Content preview
    pub preview: Option<String>,

    /// Tier of the containing CXP
    pub tier: FileTier,
}

impl SearchHit {
    /// Get full display path
    pub fn full_path(&self) -> String {
        format!("{}/{}", self.cxp_path.join("/"), self.file_path)
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Currently used bytes
    pub used_bytes: usize,

    /// Maximum allowed bytes
    pub max_bytes: usize,

    /// Number of cached CXPs
    pub cached_cxps: usize,

    /// Number of Hot CXPs in cache
    pub hot_cxps: usize,

    /// Number of Warm CXPs in cache
    pub warm_cxps: usize,

    /// Number of Cold CXPs in cache
    pub cold_cxps: usize,
}

impl MemoryStats {
    /// Get usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.max_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.max_bytes as f64) * 100.0
        }
    }

    /// Human readable used memory
    pub fn used_display(&self) -> String {
        format_bytes(self.used_bytes)
    }

    /// Human readable max memory
    pub fn max_display(&self) -> String {
        format_bytes(self.max_bytes)
    }
}

fn format_bytes(bytes: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manager_creation() {
        let config = CxpManagerConfig::default();
        let manager = CxpManager::new(config);

        let stats = manager.memory_usage().unwrap();
        assert_eq!(stats.used_bytes, 0);
        assert_eq!(stats.cached_cxps, 0);
    }

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats {
            used_bytes: 100 * 1024 * 1024, // 100 MB
            max_bytes: 500 * 1024 * 1024,  // 500 MB
            cached_cxps: 5,
            hot_cxps: 2,
            warm_cxps: 2,
            cold_cxps: 1,
        };

        assert_eq!(stats.usage_percent(), 20.0);
        assert_eq!(stats.used_display(), "100.0 MB");
        assert_eq!(stats.max_display(), "500.0 MB");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1536 * 1024), "1.5 MB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GB");
    }
}
