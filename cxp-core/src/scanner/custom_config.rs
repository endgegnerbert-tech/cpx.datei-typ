use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Custom configuration for user-specific scanning preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    /// Custom content type categories
    pub content_types: ContentTypes,
    /// User-defined importance scores (0.0 - 1.0)
    pub importance_scores: HashMap<String, f64>,
    /// Custom tier thresholds
    pub tier_thresholds: TierThresholds,
}

impl CustomConfig {
    /// Create a new custom config with defaults
    pub fn new() -> Self {
        Self {
            content_types: ContentTypes::default(),
            importance_scores: HashMap::new(),
            tier_thresholds: TierThresholds::default(),
        }
    }

    /// Set importance score for a file extension
    pub fn set_importance(&mut self, extension: String, score: f64) {
        let clamped_score = score.clamp(0.0, 1.0);
        self.importance_scores.insert(extension, clamped_score);
    }

    /// Get importance score for a file extension
    pub fn get_importance(&self, extension: &str) -> Option<f64> {
        self.importance_scores.get(extension).copied()
    }

    /// Add a file extension to a content type
    pub fn add_to_content_type(&mut self, content_type: &str, extension: String) {
        match content_type {
            "work" => self.content_types.work.push(extension),
            "archive" => self.content_types.archive.push(extension),
            "temp" => self.content_types.temp.push(extension),
            _ => {}
        }
    }

    /// Check if an extension is in the work content type
    pub fn is_work_file(&self, extension: &str) -> bool {
        self.content_types.work.contains(&extension.to_string())
    }

    /// Check if an extension is in the archive content type
    pub fn is_archive_file(&self, extension: &str) -> bool {
        self.content_types.archive.contains(&extension.to_string())
    }

    /// Check if an extension is in the temp content type
    pub fn is_temp_file(&self, extension: &str) -> bool {
        self.content_types.temp.contains(&extension.to_string())
    }
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Content type categories for files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypes {
    /// Work-related files (high priority)
    pub work: Vec<String>,
    /// Archive files (low priority)
    pub archive: Vec<String>,
    /// Temporary files (can be ignored)
    pub temp: Vec<String>,
}

impl Default for ContentTypes {
    fn default() -> Self {
        Self {
            work: vec![
                "rs".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "py".to_string(),
                "md".to_string(),
                "toml".to_string(),
                "json".to_string(),
            ],
            archive: vec![
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "7z".to_string(),
                "rar".to_string(),
            ],
            temp: vec![
                "tmp".to_string(),
                "bak".to_string(),
                "cache".to_string(),
                "log".to_string(),
            ],
        }
    }
}

/// Thresholds for tier categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierThresholds {
    /// Minimum score for HOT tier (0.0 - 1.0)
    pub hot_min: f64,
    /// Minimum score for WARM tier (0.0 - 1.0)
    /// Everything below warm_min is COLD
    pub warm_min: f64,
}

impl Default for TierThresholds {
    fn default() -> Self {
        Self {
            hot_min: 0.7,
            warm_min: 0.4,
        }
    }
}

impl TierThresholds {
    /// Create new thresholds with validation
    pub fn new(hot_min: f64, warm_min: f64) -> Result<Self, String> {
        if hot_min < 0.0 || hot_min > 1.0 {
            return Err("hot_min must be between 0.0 and 1.0".to_string());
        }
        if warm_min < 0.0 || warm_min > 1.0 {
            return Err("warm_min must be between 0.0 and 1.0".to_string());
        }
        if hot_min <= warm_min {
            return Err("hot_min must be greater than warm_min".to_string());
        }

        Ok(Self { hot_min, warm_min })
    }

    /// Determine tier based on score
    pub fn get_tier(&self, score: f64) -> crate::scanner::tier::Tier {
        if score >= self.hot_min {
            crate::scanner::tier::Tier::Hot
        } else if score >= self.warm_min {
            crate::scanner::tier::Tier::Warm
        } else {
            crate::scanner::tier::Tier::Cold
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_config_new() {
        let config = CustomConfig::new();
        assert!(config.importance_scores.is_empty());
        assert!(!config.content_types.work.is_empty());
    }

    #[test]
    fn test_set_importance() {
        let mut config = CustomConfig::new();
        config.set_importance("rs".to_string(), 0.9);

        assert_eq!(config.get_importance("rs"), Some(0.9));
    }

    #[test]
    fn test_importance_clamping() {
        let mut config = CustomConfig::new();
        config.set_importance("test".to_string(), 1.5); // Should clamp to 1.0

        assert_eq!(config.get_importance("test"), Some(1.0));

        config.set_importance("test2".to_string(), -0.5); // Should clamp to 0.0
        assert_eq!(config.get_importance("test2"), Some(0.0));
    }

    #[test]
    fn test_content_types() {
        let mut config = CustomConfig::new();

        assert!(config.is_work_file("rs"));
        assert!(!config.is_work_file("unknown"));

        config.add_to_content_type("work", "custom".to_string());
        assert!(config.is_work_file("custom"));
    }

    #[test]
    fn test_tier_thresholds_default() {
        let thresholds = TierThresholds::default();
        assert_eq!(thresholds.hot_min, 0.7);
        assert_eq!(thresholds.warm_min, 0.4);
    }

    #[test]
    fn test_tier_thresholds_validation() {
        // Valid thresholds
        assert!(TierThresholds::new(0.8, 0.5).is_ok());

        // Invalid: hot_min <= warm_min
        assert!(TierThresholds::new(0.5, 0.8).is_err());

        // Invalid: out of range
        assert!(TierThresholds::new(1.5, 0.5).is_err());
        assert!(TierThresholds::new(0.8, -0.1).is_err());
    }

    #[test]
    fn test_get_tier() {
        let thresholds = TierThresholds::default();

        assert_eq!(
            thresholds.get_tier(0.9),
            crate::scanner::tier::Tier::Hot
        );
        assert_eq!(
            thresholds.get_tier(0.5),
            crate::scanner::tier::Tier::Warm
        );
        assert_eq!(
            thresholds.get_tier(0.2),
            crate::scanner::tier::Tier::Cold
        );
    }
}
