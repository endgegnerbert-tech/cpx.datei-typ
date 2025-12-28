use std::path::Path;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::scanner::profile::UserProfile;
use crate::scanner::custom_config::CustomConfig;

/// Relevance scorer for files
#[derive(Debug)]
pub struct RelevanceScorer {
    profile: UserProfile,
    custom_config: Option<CustomConfig>,
}

impl RelevanceScorer {
    /// Create a new relevance scorer
    pub fn new(profile: UserProfile) -> Self {
        Self {
            profile,
            custom_config: None,
        }
    }

    /// Create a scorer with custom configuration
    pub fn with_config(profile: UserProfile, custom_config: CustomConfig) -> Self {
        Self {
            profile,
            custom_config: Some(custom_config),
        }
    }

    /// Calculate relevance score for a file (0.0 - 1.0)
    pub fn score_file(&self, metadata: &FileMetadata) -> f64 {
        let mut score = 0.0;

        // Extension relevance (40% weight)
        score += self.score_extension(&metadata.extension) * 0.4;

        // Recency (30% weight)
        score += self.score_recency(&metadata.modified) * 0.3;

        // Size (20% weight)
        score += self.score_size(metadata.size) * 0.2;

        // Path depth (10% weight)
        score += self.score_path_depth(metadata.path_depth) * 0.1;

        // Apply custom importance if available
        if let Some(config) = &self.custom_config {
            if let Some(importance) = config.get_importance(&metadata.extension) {
                score = score * 0.5 + importance * 0.5;
            }
        }

        score.clamp(0.0, 1.0)
    }

    /// Score based on file extension
    fn score_extension(&self, extension: &str) -> f64 {
        let config = self.profile.default_config();
        let relevant_exts: Vec<&str> = config.file_extensions.iter().map(|s| s.as_str()).collect();

        if relevant_exts.contains(&extension) {
            1.0
        } else if extension.is_empty() {
            0.3 // Files without extensions get low score
        } else {
            0.5 // Unknown extensions get medium score
        }
    }

    /// Score based on file modification time
    fn score_recency(&self, modified: &SystemTime) -> f64 {
        let now = SystemTime::now();
        let age = now
            .duration_since(*modified)
            .unwrap_or_default()
            .as_secs();

        // Score based on age
        if age < 60 * 60 * 24 {
            // < 1 day
            1.0
        } else if age < 60 * 60 * 24 * 7 {
            // < 1 week
            0.9
        } else if age < 60 * 60 * 24 * 30 {
            // < 1 month
            0.7
        } else if age < 60 * 60 * 24 * 90 {
            // < 3 months
            0.5
        } else if age < 60 * 60 * 24 * 365 {
            // < 1 year
            0.3
        } else {
            0.1 // > 1 year
        }
    }

    /// Score based on file size
    fn score_size(&self, size: u64) -> f64 {
        // Prefer medium-sized files
        // Too small (< 1KB) or too large (> 10MB) get lower scores
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;

        if size < KB {
            0.5 // Very small files
        } else if size < 100 * KB {
            1.0 // Sweet spot: 1KB - 100KB
        } else if size < MB {
            0.9 // 100KB - 1MB
        } else if size < 10 * MB {
            0.7 // 1MB - 10MB
        } else if size < 100 * MB {
            0.4 // 10MB - 100MB
        } else {
            0.2 // > 100MB
        }
    }

    /// Score based on path depth
    fn score_path_depth(&self, depth: usize) -> f64 {
        // Prefer files closer to the root
        match depth {
            0..=2 => 1.0,   // Root or 1-2 levels deep
            3..=4 => 0.8,   // 3-4 levels deep
            5..=6 => 0.6,   // 5-6 levels deep
            7..=10 => 0.4,  // 7-10 levels deep
            _ => 0.2,       // Very deep
        }
    }

    /// Get the user profile
    pub fn profile(&self) -> UserProfile {
        self.profile
    }
}

/// Metadata about a file for relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File path (relative or absolute)
    pub path: String,
    /// File extension (lowercase, without dot)
    pub extension: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: SystemTime,
    /// Path depth (number of directory separators)
    pub path_depth: usize,
}

impl FileMetadata {
    /// Create file metadata from a path
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let path_depth = path.components().count();
        let modified = metadata.modified()?;

        Ok(Self {
            path: path.to_string_lossy().to_string(),
            extension,
            size: metadata.len(),
            modified,
            path_depth,
        })
    }

    /// Create metadata manually (useful for testing)
    pub fn new(
        path: String,
        extension: String,
        size: u64,
        modified: SystemTime,
        path_depth: usize,
    ) -> Self {
        Self {
            path,
            extension,
            size,
            modified,
            path_depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_score_extension_relevant() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);
        assert_eq!(scorer.score_extension("rs"), 1.0);
        assert_eq!(scorer.score_extension("ts"), 1.0);
        assert_eq!(scorer.score_extension("py"), 1.0);
    }

    #[test]
    fn test_score_extension_irrelevant() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);
        assert_eq!(scorer.score_extension(""), 0.3);
        assert_eq!(scorer.score_extension("unknown"), 0.5);
    }

    #[test]
    fn test_score_recency() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);
        let now = SystemTime::now();

        // Recent file (1 hour ago)
        let recent = now - Duration::from_secs(60 * 60);
        assert_eq!(scorer.score_recency(&recent), 1.0);

        // 1 month old
        let month_old = now - Duration::from_secs(60 * 60 * 24 * 30);
        assert_eq!(scorer.score_recency(&month_old), 0.5);

        // Very old (2 years)
        let very_old = now - Duration::from_secs(60 * 60 * 24 * 365 * 2);
        assert_eq!(scorer.score_recency(&very_old), 0.1);
    }

    #[test]
    fn test_score_size() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);

        assert_eq!(scorer.score_size(500), 0.5); // < 1KB
        assert_eq!(scorer.score_size(50 * 1024), 1.0); // 50KB (sweet spot)
        assert_eq!(scorer.score_size(500 * 1024), 0.9); // 500KB
        assert_eq!(scorer.score_size(5 * 1024 * 1024), 0.7); // 5MB
        assert_eq!(scorer.score_size(50 * 1024 * 1024), 0.4); // 50MB
        assert_eq!(scorer.score_size(200 * 1024 * 1024), 0.2); // 200MB
    }

    #[test]
    fn test_score_path_depth() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);

        assert_eq!(scorer.score_path_depth(1), 1.0);
        assert_eq!(scorer.score_path_depth(3), 0.8);
        assert_eq!(scorer.score_path_depth(5), 0.6);
        assert_eq!(scorer.score_path_depth(8), 0.4);
        assert_eq!(scorer.score_path_depth(15), 0.2);
    }

    #[test]
    fn test_score_file() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);
        let now = SystemTime::now();

        let metadata = FileMetadata::new(
            "src/main.rs".to_string(),
            "rs".to_string(),
            10 * 1024, // 10KB
            now - Duration::from_secs(60 * 60), // 1 hour ago
            2, // path depth
        );

        let score = scorer.score_file(&metadata);
        assert!(score > 0.8); // Should be high score
        assert!(score <= 1.0);
    }

    #[test]
    fn test_score_file_low_relevance() {
        let scorer = RelevanceScorer::new(UserProfile::Developer);
        let now = SystemTime::now();

        let metadata = FileMetadata::new(
            "deep/nested/path/to/file.unknown".to_string(),
            "unknown".to_string(),
            200 * 1024 * 1024, // 200MB
            now - Duration::from_secs(60 * 60 * 24 * 365 * 2), // 2 years old
            10, // deep path
        );

        let score = scorer.score_file(&metadata);
        assert!(score < 0.5); // Should be low score
    }

    #[test]
    fn test_with_custom_config() {
        let mut config = CustomConfig::new();
        config.set_importance("custom".to_string(), 0.9);

        let scorer = RelevanceScorer::with_config(UserProfile::Developer, config);
        let now = SystemTime::now();

        let metadata = FileMetadata::new(
            "file.custom".to_string(),
            "custom".to_string(),
            10 * 1024,
            now,
            1,
        );

        let score = scorer.score_file(&metadata);
        assert!(score > 0.7); // Custom importance should boost score
    }

    #[test]
    fn test_file_metadata_new() {
        let now = SystemTime::now();
        let metadata = FileMetadata::new(
            "test.rs".to_string(),
            "rs".to_string(),
            1024,
            now,
            1,
        );

        assert_eq!(metadata.path, "test.rs");
        assert_eq!(metadata.extension, "rs");
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.path_depth, 1);
    }
}
