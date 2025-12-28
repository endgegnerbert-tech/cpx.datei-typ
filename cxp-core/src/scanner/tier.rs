use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// File tier based on relevance score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tier {
    /// High priority - recently modified, relevant files
    Hot,
    /// Medium priority - somewhat relevant or older files
    Warm,
    /// Low priority - old, rarely accessed, or irrelevant files
    Cold,
}

impl Tier {
    /// Get a human-readable name for the tier
    pub fn name(&self) -> &'static str {
        match self {
            Tier::Hot => "HOT",
            Tier::Warm => "WARM",
            Tier::Cold => "COLD",
        }
    }

    /// Get a description of the tier
    pub fn description(&self) -> &'static str {
        match self {
            Tier::Hot => "High priority - active work files",
            Tier::Warm => "Medium priority - relevant but not actively used",
            Tier::Cold => "Low priority - archived or rarely accessed",
        }
    }

    /// Get a priority score for ordering (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            Tier::Hot => 3,
            Tier::Warm => 2,
            Tier::Cold => 1,
        }
    }

    /// Get the tier from a relevance score (0.0 - 1.0)
    pub fn from_score(score: f64) -> Self {
        if score >= 0.7 {
            Tier::Hot
        } else if score >= 0.4 {
            Tier::Warm
        } else {
            Tier::Cold
        }
    }

    /// Get all tiers in priority order
    pub fn all() -> Vec<Tier> {
        vec![Tier::Hot, Tier::Warm, Tier::Cold]
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Manages file categorization into tiers
#[derive(Debug)]
pub struct TierManager {
    /// Files grouped by tier
    tiers: HashMap<Tier, Vec<String>>,
    /// Total file count
    total_files: usize,
}

impl TierManager {
    /// Create a new tier manager
    pub fn new() -> Self {
        let mut tiers = HashMap::new();
        tiers.insert(Tier::Hot, Vec::new());
        tiers.insert(Tier::Warm, Vec::new());
        tiers.insert(Tier::Cold, Vec::new());

        Self {
            tiers,
            total_files: 0,
        }
    }

    /// Add a file to a tier
    pub fn add_file(&mut self, tier: Tier, file_path: String) {
        if let Some(files) = self.tiers.get_mut(&tier) {
            files.push(file_path);
            self.total_files += 1;
        }
    }

    /// Add a file with automatic tier assignment based on score
    pub fn add_file_with_score(&mut self, file_path: String, score: f64) {
        let tier = Tier::from_score(score);
        self.add_file(tier, file_path);
    }

    /// Get files in a specific tier
    pub fn get_files(&self, tier: Tier) -> &[String] {
        self.tiers.get(&tier).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get file count in a specific tier
    pub fn count(&self, tier: Tier) -> usize {
        self.tiers.get(&tier).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total file count across all tiers
    pub fn total_count(&self) -> usize {
        self.total_files
    }

    /// Get tier statistics
    pub fn stats(&self) -> TierStats {
        TierStats {
            hot_count: self.count(Tier::Hot),
            warm_count: self.count(Tier::Warm),
            cold_count: self.count(Tier::Cold),
            total: self.total_files,
        }
    }

    /// Get percentage distribution across tiers
    pub fn distribution(&self) -> TierDistribution {
        let total = self.total_files as f64;
        if total == 0.0 {
            return TierDistribution {
                hot_percent: 0.0,
                warm_percent: 0.0,
                cold_percent: 0.0,
            };
        }

        TierDistribution {
            hot_percent: (self.count(Tier::Hot) as f64 / total) * 100.0,
            warm_percent: (self.count(Tier::Warm) as f64 / total) * 100.0,
            cold_percent: (self.count(Tier::Cold) as f64 / total) * 100.0,
        }
    }

    /// Clear all files from all tiers
    pub fn clear(&mut self) {
        for files in self.tiers.values_mut() {
            files.clear();
        }
        self.total_files = 0;
    }

    /// Get all files sorted by tier priority
    pub fn get_all_sorted(&self) -> Vec<String> {
        let mut all_files = Vec::with_capacity(self.total_files);

        // Add in priority order: Hot -> Warm -> Cold
        for tier in [Tier::Hot, Tier::Warm, Tier::Cold] {
            if let Some(files) = self.tiers.get(&tier) {
                all_files.extend(files.clone());
            }
        }

        all_files
    }
}

impl Default for TierManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about tier distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierStats {
    pub hot_count: usize,
    pub warm_count: usize,
    pub cold_count: usize,
    pub total: usize,
}

impl TierStats {
    /// Check if the distribution is balanced
    pub fn is_balanced(&self) -> bool {
        if self.total == 0 {
            return true;
        }

        // Consider balanced if no tier has > 60% of files
        let max_percent = 0.6;
        let total = self.total as f64;

        self.hot_count as f64 / total < max_percent
            && self.warm_count as f64 / total < max_percent
            && self.cold_count as f64 / total < max_percent
    }
}

/// Percentage distribution across tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDistribution {
    pub hot_percent: f64,
    pub warm_percent: f64,
    pub cold_percent: f64,
}

impl TierDistribution {
    /// Check if the distribution is valid (should sum to ~100%)
    pub fn is_valid(&self) -> bool {
        let sum = self.hot_percent + self.warm_percent + self.cold_percent;
        (sum - 100.0).abs() < 1.0 // Allow 1% tolerance for floating point errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_names() {
        assert_eq!(Tier::Hot.name(), "HOT");
        assert_eq!(Tier::Warm.name(), "WARM");
        assert_eq!(Tier::Cold.name(), "COLD");
    }

    #[test]
    fn test_tier_priority() {
        assert!(Tier::Hot.priority() > Tier::Warm.priority());
        assert!(Tier::Warm.priority() > Tier::Cold.priority());
    }

    #[test]
    fn test_tier_from_score() {
        assert_eq!(Tier::from_score(0.9), Tier::Hot);
        assert_eq!(Tier::from_score(0.7), Tier::Hot);
        assert_eq!(Tier::from_score(0.5), Tier::Warm);
        assert_eq!(Tier::from_score(0.4), Tier::Warm);
        assert_eq!(Tier::from_score(0.2), Tier::Cold);
        assert_eq!(Tier::from_score(0.0), Tier::Cold);
    }

    #[test]
    fn test_tier_all() {
        let tiers = Tier::all();
        assert_eq!(tiers.len(), 3);
        assert_eq!(tiers[0], Tier::Hot);
        assert_eq!(tiers[1], Tier::Warm);
        assert_eq!(tiers[2], Tier::Cold);
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(format!("{}", Tier::Hot), "HOT");
        assert_eq!(format!("{}", Tier::Warm), "WARM");
        assert_eq!(format!("{}", Tier::Cold), "COLD");
    }

    #[test]
    fn test_tier_manager_new() {
        let manager = TierManager::new();
        assert_eq!(manager.total_count(), 0);
        assert_eq!(manager.count(Tier::Hot), 0);
        assert_eq!(manager.count(Tier::Warm), 0);
        assert_eq!(manager.count(Tier::Cold), 0);
    }

    #[test]
    fn test_add_file() {
        let mut manager = TierManager::new();
        manager.add_file(Tier::Hot, "file1.rs".to_string());
        manager.add_file(Tier::Hot, "file2.rs".to_string());
        manager.add_file(Tier::Warm, "file3.txt".to_string());

        assert_eq!(manager.count(Tier::Hot), 2);
        assert_eq!(manager.count(Tier::Warm), 1);
        assert_eq!(manager.count(Tier::Cold), 0);
        assert_eq!(manager.total_count(), 3);
    }

    #[test]
    fn test_add_file_with_score() {
        let mut manager = TierManager::new();
        manager.add_file_with_score("hot.rs".to_string(), 0.9);
        manager.add_file_with_score("warm.txt".to_string(), 0.5);
        manager.add_file_with_score("cold.log".to_string(), 0.2);

        assert_eq!(manager.count(Tier::Hot), 1);
        assert_eq!(manager.count(Tier::Warm), 1);
        assert_eq!(manager.count(Tier::Cold), 1);
    }

    #[test]
    fn test_get_files() {
        let mut manager = TierManager::new();
        manager.add_file(Tier::Hot, "file1.rs".to_string());
        manager.add_file(Tier::Hot, "file2.rs".to_string());

        let hot_files = manager.get_files(Tier::Hot);
        assert_eq!(hot_files.len(), 2);
        assert!(hot_files.contains(&"file1.rs".to_string()));
        assert!(hot_files.contains(&"file2.rs".to_string()));
    }

    #[test]
    fn test_stats() {
        let mut manager = TierManager::new();
        manager.add_file(Tier::Hot, "file1.rs".to_string());
        manager.add_file(Tier::Warm, "file2.txt".to_string());
        manager.add_file(Tier::Cold, "file3.log".to_string());

        let stats = manager.stats();
        assert_eq!(stats.hot_count, 1);
        assert_eq!(stats.warm_count, 1);
        assert_eq!(stats.cold_count, 1);
        assert_eq!(stats.total, 3);
    }

    #[test]
    fn test_distribution() {
        let mut manager = TierManager::new();
        manager.add_file(Tier::Hot, "file1.rs".to_string());
        manager.add_file(Tier::Hot, "file2.rs".to_string());
        manager.add_file(Tier::Warm, "file3.txt".to_string());
        manager.add_file(Tier::Cold, "file4.log".to_string());

        let dist = manager.distribution();
        assert_eq!(dist.hot_percent, 50.0);
        assert_eq!(dist.warm_percent, 25.0);
        assert_eq!(dist.cold_percent, 25.0);
        assert!(dist.is_valid());
    }

    #[test]
    fn test_distribution_empty() {
        let manager = TierManager::new();
        let dist = manager.distribution();
        assert_eq!(dist.hot_percent, 0.0);
        assert_eq!(dist.warm_percent, 0.0);
        assert_eq!(dist.cold_percent, 0.0);
    }

    #[test]
    fn test_clear() {
        let mut manager = TierManager::new();
        manager.add_file(Tier::Hot, "file1.rs".to_string());
        manager.add_file(Tier::Warm, "file2.txt".to_string());

        assert_eq!(manager.total_count(), 2);

        manager.clear();
        assert_eq!(manager.total_count(), 0);
        assert_eq!(manager.count(Tier::Hot), 0);
        assert_eq!(manager.count(Tier::Warm), 0);
    }

    #[test]
    fn test_get_all_sorted() {
        let mut manager = TierManager::new();
        manager.add_file(Tier::Cold, "cold.log".to_string());
        manager.add_file(Tier::Hot, "hot.rs".to_string());
        manager.add_file(Tier::Warm, "warm.txt".to_string());

        let sorted = manager.get_all_sorted();
        assert_eq!(sorted.len(), 3);
        // Should be in priority order: Hot, Warm, Cold
        assert_eq!(sorted[0], "hot.rs");
        assert_eq!(sorted[1], "warm.txt");
        assert_eq!(sorted[2], "cold.log");
    }

    #[test]
    fn test_stats_is_balanced() {
        let balanced = TierStats {
            hot_count: 30,
            warm_count: 30,
            cold_count: 40,
            total: 100,
        };
        assert!(balanced.is_balanced());

        let unbalanced = TierStats {
            hot_count: 70,
            warm_count: 20,
            cold_count: 10,
            total: 100,
        };
        assert!(!unbalanced.is_balanced());
    }
}
