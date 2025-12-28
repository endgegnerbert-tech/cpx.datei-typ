use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration for file scanning operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Paths to scan
    pub paths: Vec<PathBuf>,
    /// File extensions to include
    pub file_extensions: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Include image files in scan
    pub include_images: bool,
    /// Include hidden files (starting with .)
    pub include_hidden: bool,
    /// Custom ignore patterns (gitignore-style)
    pub custom_ignore: Vec<String>,
    /// Patterns to force include despite ignore rules
    pub force_include: Vec<String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            file_extensions: vec![],
            max_file_size: 10 * 1024 * 1024, // 10 MB
            include_images: false,
            include_hidden: false,
            custom_ignore: vec![],
            force_include: vec![],
        }
    }
}

impl ScanConfig {
    /// Create a new ScanConfig with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a path to scan
    pub fn add_path(mut self, path: PathBuf) -> Self {
        self.paths.push(path);
        self
    }

    /// Add multiple paths to scan
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.paths = paths;
        self
    }

    /// Add a file extension to filter
    pub fn add_extension(mut self, ext: String) -> Self {
        self.file_extensions.push(ext);
        self
    }

    /// Set file extensions to filter
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.file_extensions = extensions;
        self
    }

    /// Set maximum file size
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Enable/disable image inclusion
    pub fn with_images(mut self, include: bool) -> Self {
        self.include_images = include;
        self
    }

    /// Enable/disable hidden files
    pub fn with_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }

    /// Add custom ignore patterns
    pub fn with_custom_ignore(mut self, patterns: Vec<String>) -> Self {
        self.custom_ignore = patterns;
        self
    }

    /// Add force include patterns
    pub fn with_force_include(mut self, patterns: Vec<String>) -> Self {
        self.force_include = patterns;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScanConfig::default();
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert!(!config.include_images);
        assert!(!config.include_hidden);
        assert!(config.paths.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let config = ScanConfig::new()
            .add_path(PathBuf::from("/test"))
            .add_extension("rs".to_string())
            .with_max_file_size(5 * 1024 * 1024)
            .with_images(true);

        assert_eq!(config.paths.len(), 1);
        assert_eq!(config.file_extensions.len(), 1);
        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert!(config.include_images);
    }
}
