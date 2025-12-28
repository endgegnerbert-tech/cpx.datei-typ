use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

/// Patterns that are ALWAYS ignored - no override possible
pub const ALWAYS_IGNORE: &[&str] = &[
    // === DEPENDENCIES ===
    "node_modules/**",
    "**/node_modules/**",
    "vendor/**",
    ".venv/**",
    "venv/**",
    "__pycache__/**",
    "**/__pycache__/**",
    ".tox/**",
    "*.egg-info/**",
    // === BUILD OUTPUT ===
    "target/debug/**",
    "target/release/**",
    "dist/**",
    "build/**",
    ".next/**",
    ".nuxt/**",
    ".output/**",
    "out/**",
    // === GIT INTERNALS ===
    ".git/objects/**",
    ".git/lfs/**",
    // === CACHES ===
    ".cache/**",
    "**/.cache/**",
    ".pytest_cache/**",
    ".mypy_cache/**",
    ".ruff_cache/**",
    "*.pyc",
    "*.pyo",
    ".sass-cache/**",
    ".parcel-cache/**",
    // === IDE ===
    ".idea/**",
    ".vs/**",
    "*.swp",
    "*.swo",
    "*~",
    // === BINARIES ===
    "*.exe",
    "*.dll",
    "*.so",
    "*.dylib",
    "*.wasm",
    "*.o",
    "*.obj",
    "*.a",
    "*.lib",
    // === ARCHIVES ===
    "*.zip",
    "*.tar",
    "*.gz",
    "*.bz2",
    "*.xz",
    "*.rar",
    "*.7z",
    "*.dmg",
    "*.iso",
    // === LOGS ===
    "*.log",
    "logs/**",
    "*.log.*",
    // === LOCK FILES ===
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "poetry.lock",
    "Pipfile.lock",
    // === GENERATED ===
    "*.min.js",
    "*.min.css",
    "*.map",
    "*.d.ts",
    "*.generated.*",
    // === SYSTEM ===
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",
];

/// Patterns ignored by default but can be overridden
pub const DEFAULT_IGNORE: &[&str] = &[
    ".vscode/**",
    ".github/**",
    "docs/**",
    "test/**",
    "tests/**",
    "spec/**",
    "__tests__/**",
    "*.test.*",
    "*.spec.*",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    /// Custom patterns to ignore (in addition to ALWAYS_IGNORE and DEFAULT_IGNORE)
    pub custom_ignore: Vec<String>,
    /// Patterns to force include (overrides DEFAULT_IGNORE and custom_ignore, but NOT ALWAYS_IGNORE)
    pub force_include: Vec<String>,
    /// Maximum file size in bytes (files larger than this are ignored)
    pub max_file_size: u64,
    /// Whether to include hidden files/directories (starting with .)
    pub include_hidden: bool,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            custom_ignore: vec![],
            force_include: vec![],
            max_file_size: 10 * 1024 * 1024, // 10 MB
            include_hidden: false,
        }
    }
}

impl IgnoreConfig {
    /// Create a new IgnoreConfig with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom ignore pattern
    pub fn add_ignore<S: Into<String>>(mut self, pattern: S) -> Self {
        self.custom_ignore.push(pattern.into());
        self
    }

    /// Add a force include pattern
    pub fn add_force_include<S: Into<String>>(mut self, pattern: S) -> Self {
        self.force_include.push(pattern.into());
        self
    }

    /// Set maximum file size
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set whether to include hidden files
    pub fn with_include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }

    /// Build the ALWAYS_IGNORE GlobSet (cannot be overridden)
    pub fn build_always_ignore_set(&self) -> Result<GlobSet> {
        build_globset(ALWAYS_IGNORE)
    }

    /// Build the default ignore GlobSet (can be overridden by force_include)
    pub fn build_default_ignore_set(&self) -> Result<GlobSet> {
        build_globset(DEFAULT_IGNORE)
    }

    /// Build the custom ignore GlobSet (user-defined patterns)
    pub fn build_custom_ignore_set(&self) -> Result<GlobSet> {
        let patterns: Vec<&str> = self.custom_ignore.iter().map(|s| s.as_str()).collect();
        build_globset(&patterns)
    }

    /// Build the force include GlobSet (overrides DEFAULT_IGNORE and custom_ignore)
    pub fn build_force_include_set(&self) -> Result<GlobSet> {
        let patterns: Vec<&str> = self.force_include.iter().map(|s| s.as_str()).collect();
        build_globset(&patterns)
    }

    /// Check if a file path should be ignored
    /// Returns true if the file should be ignored
    pub fn should_ignore(&self, path: &str) -> Result<bool> {
        let always_ignore = self.build_always_ignore_set()?;
        let default_ignore = self.build_default_ignore_set()?;
        let custom_ignore = self.build_custom_ignore_set()?;
        let force_include = self.build_force_include_set()?;

        // ALWAYS_IGNORE has highest priority - cannot be overridden
        if always_ignore.is_match(path) {
            return Ok(true);
        }

        // force_include overrides DEFAULT_IGNORE and custom_ignore
        if force_include.is_match(path) {
            return Ok(false);
        }

        // Check DEFAULT_IGNORE and custom_ignore
        if default_ignore.is_match(path) || custom_ignore.is_match(path) {
            return Ok(true);
        }

        // Check hidden files if not included
        if !self.include_hidden && is_hidden_path(path) {
            return Ok(true);
        }

        Ok(false)
    }
}

/// Build a GlobSet from a slice of pattern strings
pub fn build_globset(patterns: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let glob = Glob::new(pattern)
            .with_context(|| format!("Failed to compile glob pattern: {}", pattern))?;
        builder.add(glob);
    }

    builder
        .build()
        .context("Failed to build GlobSet from patterns")
}

/// Check if a path represents a hidden file or directory
/// (starts with . but is not . or ..)
fn is_hidden_path(path: &str) -> bool {
    path.split('/')
        .any(|component| component.starts_with('.') && component != "." && component != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_ignore_patterns() {
        let config = IgnoreConfig::default();

        // Should ignore node_modules
        assert!(config.should_ignore("node_modules/package/file.js").unwrap());
        assert!(config.should_ignore("src/node_modules/file.js").unwrap());

        // Should ignore build output
        assert!(config.should_ignore("target/debug/binary").unwrap());
        assert!(config.should_ignore("dist/bundle.js").unwrap());
        assert!(config.should_ignore(".next/cache/file").unwrap());

        // Should ignore binaries
        assert!(config.should_ignore("app.exe").unwrap());
        assert!(config.should_ignore("lib.so").unwrap());
        assert!(config.should_ignore("binary.wasm").unwrap());

        // Should ignore lock files
        assert!(config.should_ignore("package-lock.json").unwrap());
        assert!(config.should_ignore("Cargo.lock").unwrap());
    }

    #[test]
    fn test_default_ignore_patterns() {
        let config = IgnoreConfig::default();

        // Should ignore test files by default
        assert!(config.should_ignore("tests/unit.rs").unwrap());
        assert!(config.should_ignore("src/component.test.ts").unwrap());
        assert!(config.should_ignore(".vscode/settings.json").unwrap());
    }

    #[test]
    fn test_force_include_overrides() {
        let config = IgnoreConfig::default().add_force_include("tests/**");

        // Should NOT ignore tests/ because of force_include
        assert!(!config.should_ignore("tests/unit.rs").unwrap());

        // But should still ignore ALWAYS_IGNORE patterns
        assert!(config.should_ignore("node_modules/file.js").unwrap());
    }

    #[test]
    fn test_custom_ignore() {
        let config = IgnoreConfig::default().add_ignore("*.tmp");

        assert!(config.should_ignore("file.tmp").unwrap());
        assert!(!config.should_ignore("file.txt").unwrap());
    }

    #[test]
    fn test_hidden_files() {
        let config_default = IgnoreConfig::default();
        let config_include_hidden = IgnoreConfig::default().with_include_hidden(true);

        // By default, hidden files are ignored
        assert!(config_default.should_ignore(".hidden").unwrap());
        assert!(config_default.should_ignore("dir/.hidden").unwrap());

        // With include_hidden = true, they are not ignored
        assert!(!config_include_hidden.should_ignore(".hidden").unwrap());
        assert!(!config_include_hidden.should_ignore("dir/.hidden").unwrap());

        // But ALWAYS_IGNORE patterns still apply
        assert!(config_include_hidden.should_ignore(".git/objects/file").unwrap());
    }

    #[test]
    fn test_is_hidden_path() {
        assert!(is_hidden_path(".hidden"));
        assert!(is_hidden_path("dir/.hidden"));
        assert!(is_hidden_path(".config/file"));

        assert!(!is_hidden_path("normal"));
        assert!(!is_hidden_path("dir/normal"));
        assert!(!is_hidden_path("."));
        assert!(!is_hidden_path(".."));
    }

    #[test]
    fn test_build_globset() {
        let patterns = &["*.txt", "*.md", "docs/**"];
        let globset = build_globset(patterns).unwrap();

        assert!(globset.is_match("file.txt"));
        assert!(globset.is_match("README.md"));
        assert!(globset.is_match("docs/guide.md"));
        assert!(!globset.is_match("file.rs"));
    }

    #[test]
    fn test_max_file_size() {
        let config = IgnoreConfig::default().with_max_file_size(1024);
        assert_eq!(config.max_file_size, 1024);
    }

    #[test]
    fn test_always_ignore_cannot_be_overridden() {
        let config = IgnoreConfig::default()
            .add_force_include("node_modules/**")
            .add_force_include("*.exe");

        // Even with force_include, ALWAYS_IGNORE patterns are still ignored
        assert!(config.should_ignore("node_modules/file.js").unwrap());
        assert!(config.should_ignore("app.exe").unwrap());
    }
}
