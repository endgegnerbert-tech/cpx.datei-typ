//! Recursive CXP Builder
//!
//! Creates hierarchical CXP structures from directory trees.
//! Automatically splits large directories into child CXPs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::Utc;
use walkdir::WalkDir;

use crate::recursive::{CxpRef, CxpStorage, CxpRefMeta, FileTier, ChildrenMap};
use crate::global_index::{GlobalIndex, GlobalIndexEntry};
use crate::format::CxpBuilder;
use crate::manifest::Manifest;
use crate::{Result, CxpError};

/// Configuration for recursive CXP building
#[derive(Debug, Clone)]
pub struct RecursiveBuildConfig {
    /// Minimum size (bytes) to create a separate child CXP
    pub min_size_for_child: u64,

    /// Minimum file count to create a separate child CXP
    pub min_files_for_child: usize,

    /// Maximum recursion depth
    pub max_depth: usize,

    /// Output directory for CXP files
    pub output_dir: PathBuf,

    /// Directories to ignore
    pub ignored_dirs: Vec<String>,

    /// File extensions to include
    pub include_extensions: Option<Vec<String>>,

    /// Extract keywords from content
    pub extract_keywords: bool,
}

impl Default for RecursiveBuildConfig {
    fn default() -> Self {
        Self {
            min_size_for_child: 10 * 1024 * 1024, // 10 MB
            min_files_for_child: 50,
            max_depth: 10,
            output_dir: PathBuf::from("~/.contextai/"),
            ignored_dirs: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "__pycache__".to_string(),
                ".cache".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "vendor".to_string(),
                ".next".to_string(),
                "venv".to_string(),
                ".venv".to_string(),
            ],
            include_extensions: None,
            extract_keywords: true,
        }
    }
}

/// Statistics about a directory
#[derive(Debug, Clone, Default)]
pub struct DirStats {
    /// Total file count
    pub file_count: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Subdirectory count
    pub subdir_count: usize,
    /// File types found
    pub file_types: HashMap<String, usize>,
    /// Deepest file modification time
    pub newest_modified: Option<std::time::SystemTime>,
}

/// Structure proposal for a directory
#[derive(Debug)]
pub struct ProposedStructure {
    /// Name of this node
    pub name: String,
    /// Should be a separate CXP?
    pub should_be_cxp: bool,
    /// Stats for this directory
    pub stats: DirStats,
    /// Child proposals
    pub children: Vec<ProposedStructure>,
    /// Files to include directly
    pub direct_files: Vec<PathBuf>,
    /// Estimated tier
    pub tier: FileTier,
}

/// Builder for recursive CXP hierarchies
pub struct RecursiveBuilder {
    /// Configuration
    config: RecursiveBuildConfig,
    /// Global index for all built CXPs
    global_index: GlobalIndex,
    /// Built CXP paths
    built_cxps: Vec<PathBuf>,
}

impl RecursiveBuilder {
    /// Create a new recursive builder
    pub fn new(config: RecursiveBuildConfig) -> Self {
        Self {
            config,
            global_index: GlobalIndex::new(),
            built_cxps: Vec::new(),
        }
    }

    /// Analyze a directory and propose a structure
    pub fn analyze(&self, root: &Path) -> Result<ProposedStructure> {
        self.analyze_dir(root, 0)
    }

    /// Analyze a directory recursively
    fn analyze_dir(&self, path: &Path, depth: usize) -> Result<ProposedStructure> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string();

        let mut stats = DirStats::default();
        let mut children = Vec::new();
        let mut direct_files = Vec::new();

        // Scan directory
        for entry in std::fs::read_dir(path)
            .map_err(|e| CxpError::Io(e.to_string()))?
        {
            let entry = entry.map_err(|e| CxpError::Io(e.to_string()))?;
            let entry_path = entry.path();
            let entry_name = entry_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Skip ignored directories
            if entry_path.is_dir() && self.config.ignored_dirs.contains(&entry_name.to_string()) {
                continue;
            }

            if entry_path.is_dir() {
                stats.subdir_count += 1;

                // Recursively analyze if not at max depth
                if depth < self.config.max_depth {
                    let child_structure = self.analyze_dir(&entry_path, depth + 1)?;

                    // Merge stats
                    stats.file_count += child_structure.stats.file_count;
                    stats.total_size += child_structure.stats.total_size;
                    for (ext, count) in &child_structure.stats.file_types {
                        *stats.file_types.entry(ext.clone()).or_insert(0) += count;
                    }

                    children.push(child_structure);
                }
            } else if entry_path.is_file() {
                let metadata = entry_path.metadata()
                    .map_err(|e| CxpError::Io(e.to_string()))?;

                stats.file_count += 1;
                stats.total_size += metadata.len();

                // Track file type
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    *stats.file_types.entry(ext.to_lowercase()).or_insert(0) += 1;
                }

                // Track newest modification
                if let Ok(modified) = metadata.modified() {
                    match stats.newest_modified {
                        Some(current) if modified > current => {
                            stats.newest_modified = Some(modified);
                        }
                        None => {
                            stats.newest_modified = Some(modified);
                        }
                        _ => {}
                    }
                }

                // Check if we should include this file
                let should_include = if let Some(ref extensions) = self.config.include_extensions {
                    entry_path.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| extensions.contains(&e.to_lowercase()))
                        .unwrap_or(false)
                } else {
                    crate::is_text_file(
                        entry_path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                    )
                };

                if should_include {
                    direct_files.push(entry_path);
                }
            }
        }

        // Determine if this should be a separate CXP
        let should_be_cxp = stats.total_size >= self.config.min_size_for_child
            || stats.file_count >= self.config.min_files_for_child;

        // Calculate tier based on modification time
        let tier = self.calculate_tier_from_stats(&stats);

        Ok(ProposedStructure {
            name,
            should_be_cxp,
            stats,
            children,
            direct_files,
            tier,
        })
    }

    /// Calculate tier from stats
    fn calculate_tier_from_stats(&self, stats: &DirStats) -> FileTier {
        let now = std::time::SystemTime::now();

        if let Some(newest) = stats.newest_modified {
            if let Ok(duration) = now.duration_since(newest) {
                let days = duration.as_secs() / (24 * 60 * 60);
                return match days {
                    0..=7 => FileTier::Hot,
                    8..=30 => FileTier::Warm,
                    _ => FileTier::Cold,
                };
            }
        }

        FileTier::Warm
    }

    /// Build CXPs from a proposed structure
    pub fn build(&mut self, root: &Path, structure: &ProposedStructure, parent_path: Vec<String>) -> Result<CxpRef> {
        let cxp_name = &structure.name;
        let cxp_path = self.config.output_dir.join(parent_path.join("/")).join(format!("{}.cxp", cxp_name));

        // Ensure parent directory exists
        if let Some(parent) = cxp_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CxpError::Io(e.to_string()))?;
        }

        // Build the CXP using the standard builder
        let mut builder = CxpBuilder::new(root);
        builder.scan()?;

        // Add files from this level
        for file in &structure.direct_files {
            // The standard builder's scan() already includes files
            // Here we could add custom processing
        }

        // Process children
        let mut children_map = ChildrenMap::new();
        let mut current_path = parent_path.clone();
        current_path.push(cxp_name.clone());

        for child in &structure.children {
            if child.should_be_cxp {
                // Build child CXP recursively
                let child_root = root.join(&child.name);
                let child_ref = self.build(&child_root, child, current_path.clone())?;
                children_map.add(child_ref);
            }
            // If child doesn't need its own CXP, its files are already included in parent
        }

        // Create the CXP file
        builder.build(&cxp_path)?;
        self.built_cxps.push(cxp_path.clone());

        // Create CxpRef for this CXP
        let cxp_ref = CxpRef {
            id: cxp_name.clone(),
            name: cxp_name.clone(),
            storage: CxpStorage::External { path: cxp_path },
            meta: CxpRefMeta {
                description: None,
                total_files: structure.stats.file_count,
                child_count: children_map.len(),
                has_children: !children_map.is_empty(),
                size_bytes: 0, // Will be updated after build
                original_size_bytes: structure.stats.total_size,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                category: None,
                file_types: structure.stats.file_types.keys().cloned().collect(),
                keywords: Vec::new(),
                has_embeddings: false,
            },
            last_accessed: None,
            tier: structure.tier,
            tags: Vec::new(),
        };

        // Add to global index
        self.add_to_index(&cxp_ref, &structure)?;

        Ok(cxp_ref)
    }

    /// Add CXP entries to the global index
    fn add_to_index(&mut self, cxp_ref: &CxpRef, structure: &ProposedStructure) -> Result<()> {
        let cxp_path = match &cxp_ref.storage {
            CxpStorage::External { path } => path.to_string_lossy().to_string(),
            _ => cxp_ref.id.clone(),
        };

        for file in &structure.direct_files {
            let file_name = file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let file_type = file.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let entry = GlobalIndexEntry::new(
                &cxp_ref.id,
                vec![cxp_ref.name.clone()],
                file.to_string_lossy(),
                &file_type,
            );

            self.global_index.add(entry);
        }

        Ok(())
    }

    /// Get the global index
    pub fn global_index(&self) -> &GlobalIndex {
        &self.global_index
    }

    /// Get built CXP paths
    pub fn built_cxps(&self) -> &[PathBuf] {
        &self.built_cxps
    }

    /// Build a master CXP that references all others
    pub fn build_master(&self, name: &str) -> Result<PathBuf> {
        let master_path = self.config.output_dir.join(format!("{}.cxp", name));

        // Create manifest with children
        let mut manifest = Manifest::new();
        manifest.tier = FileTier::Hot; // Master is always hot

        // Add all built CXPs as children
        // In a real implementation, we'd load each CXP's metadata

        // Serialize manifest
        let manifest_data = manifest.to_msgpack()?;

        // Create ZIP file
        let file = std::fs::File::create(&master_path)
            .map_err(|e| CxpError::Io(e.to_string()))?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Zstd);

        zip.start_file("manifest.msgpack", options)
            .map_err(|e| CxpError::Io(e.to_string()))?;

        use std::io::Write;
        zip.write_all(&manifest_data)
            .map_err(|e| CxpError::Io(e.to_string()))?;

        // Write global index
        let index_data = self.global_index.to_msgpack()?;
        zip.start_file("global_index.msgpack", options)
            .map_err(|e| CxpError::Io(e.to_string()))?;
        zip.write_all(&index_data)
            .map_err(|e| CxpError::Io(e.to_string()))?;

        zip.finish()
            .map_err(|e| CxpError::Io(e.to_string()))?;

        Ok(master_path)
    }
}

/// Known project patterns for automatic detection
#[derive(Debug, Clone)]
pub enum ProjectPattern {
    /// Node.js project (package.json)
    NodeProject,
    /// Rust project (Cargo.toml)
    RustProject,
    /// Python project (requirements.txt, pyproject.toml)
    PythonProject,
    /// Go project (go.mod)
    GoProject,
    /// Generic source directory
    SourceDirectory,
}

impl ProjectPattern {
    /// Detect project pattern from a directory
    pub fn detect(path: &Path) -> Option<ProjectPattern> {
        if path.join("package.json").exists() {
            Some(ProjectPattern::NodeProject)
        } else if path.join("Cargo.toml").exists() {
            Some(ProjectPattern::RustProject)
        } else if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
            Some(ProjectPattern::PythonProject)
        } else if path.join("go.mod").exists() {
            Some(ProjectPattern::GoProject)
        } else if path.join("src").is_dir() {
            Some(ProjectPattern::SourceDirectory)
        } else {
            None
        }
    }

    /// Get directories to ignore for this pattern
    pub fn ignored_dirs(&self) -> Vec<&'static str> {
        match self {
            ProjectPattern::NodeProject => vec!["node_modules", "dist", "build", ".next"],
            ProjectPattern::RustProject => vec!["target"],
            ProjectPattern::PythonProject => vec!["venv", ".venv", "__pycache__", ".pytest_cache"],
            ProjectPattern::GoProject => vec!["vendor"],
            ProjectPattern::SourceDirectory => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_defaults() {
        let config = RecursiveBuildConfig::default();
        assert_eq!(config.min_files_for_child, 50);
        assert_eq!(config.max_depth, 10);
        assert!(config.ignored_dirs.contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_tier_calculation() {
        let builder = RecursiveBuilder::new(RecursiveBuildConfig::default());

        // Recent modification -> Hot
        let mut stats = DirStats::default();
        stats.newest_modified = Some(std::time::SystemTime::now());
        assert_eq!(builder.calculate_tier_from_stats(&stats), FileTier::Hot);

        // No modification info -> Warm (default)
        let stats = DirStats::default();
        assert_eq!(builder.calculate_tier_from_stats(&stats), FileTier::Warm);
    }

    #[test]
    fn test_project_pattern_detection() {
        let temp_dir = TempDir::new().unwrap();

        // No pattern
        assert!(ProjectPattern::detect(temp_dir.path()).is_none());

        // Create package.json
        std::fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        assert!(matches!(
            ProjectPattern::detect(temp_dir.path()),
            Some(ProjectPattern::NodeProject)
        ));
    }
}
