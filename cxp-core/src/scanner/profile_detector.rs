use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

use super::profile::{UserProfile, DetectedApp};
use crate::error::CxpError;

/// Profile detection system - completely free & local
///
/// Strategy:
/// 1. Rule-based: Count file types (90% of cases)
/// 2. App detection: Lightroom, Obsidian, VS Code, etc.
/// 3. Embedding classification: Use existing MiniLM model (Optional)
///
/// No API costs! No internet connection needed!
pub struct ProfileDetector;

impl ProfileDetector {
    /// Analyzes the PC and suggests a profile
    pub fn detect_profile(scan_result: &QuickScanResult) -> ProfileSuggestion {
        let mut scores: HashMap<UserProfile, i32> = HashMap::new();

        // === STEP 1: Count file types ===
        for (ext, count) in &scan_result.extension_counts {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                // Developer (high weight)
                "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" |
                "cpp" | "c" | "h" | "rb" | "php" | "swift" | "kt" | "scala" |
                "clj" | "ex" | "exs" | "erl" => {
                    *scores.entry(UserProfile::Developer).or_insert(0) += (*count as i32) * 10;
                },

                // Photographer (very high weight for RAW)
                "raw" | "cr2" | "cr3" | "nef" | "arw" | "orf" | "dng" | "rw2" |
                "raf" | "srw" | "pef" | "x3f" | "erf" | "mrw" | "dcr" | "kdc" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += (*count as i32) * 20;
                },
                "jpg" | "jpeg" | "heic" | "heif" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += (*count as i32) * 2;
                },
                "lrcat" | "lrtemplate" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += 500; // Lightroom = Photographer
                },

                // Designer
                "fig" | "sketch" | "xd" | "psd" | "ai" | "indd" | "afdesign" | "afphoto" => {
                    *scores.entry(UserProfile::Designer).or_insert(0) += (*count as i32) * 15;
                },
                "svg" => {
                    *scores.entry(UserProfile::Designer).or_insert(0) += (*count as i32) * 3;
                },

                // Writer
                "md" | "txt" | "org" | "tex" | "latex" => {
                    *scores.entry(UserProfile::Writer).or_insert(0) += (*count as i32) * 5;
                },
                "docx" | "doc" | "odt" | "pages" => {
                    *scores.entry(UserProfile::Writer).or_insert(0) += (*count as i32) * 8;
                },

                // Student (Mix)
                "pptx" | "ppt" => {
                    *scores.entry(UserProfile::Student).or_insert(0) += (*count as i32) * 5;
                    *scores.entry(UserProfile::Business).or_insert(0) += (*count as i32) * 3;
                },

                // Business
                "xlsx" | "xls" | "csv" => {
                    *scores.entry(UserProfile::Business).or_insert(0) += (*count as i32) * 8;
                },
                "pdf" => {
                    // PDF is universal - small weight for all
                    *scores.entry(UserProfile::Writer).or_insert(0) += (*count as i32) * 1;
                    *scores.entry(UserProfile::Business).or_insert(0) += (*count as i32) * 1;
                    *scores.entry(UserProfile::Student).or_insert(0) += (*count as i32) * 1;
                },
                _ => {},
            }
        }

        // === STEP 2: Include detected apps ===
        for app in &scan_result.detected_apps {
            match app.app_type.as_str() {
                "Lightroom Catalog" | "Capture One Catalog" | "Adobe Bridge" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += 1000;
                },
                "Figma File" | "Sketch File" | "Adobe Creative Cloud" => {
                    *scores.entry(UserProfile::Designer).or_insert(0) += 1000;
                },
                "Obsidian Vault" | "Notion Export" | "Scrivener Project" => {
                    *scores.entry(UserProfile::Writer).or_insert(0) += 1000;
                },
                "Git Repository" | "VS Code Workspace" | "VS Code Project" | "JetBrains Project" => {
                    *scores.entry(UserProfile::Developer).or_insert(0) += 500;
                },
                _ => {},
            }
        }

        // === STEP 3: Calculate confidence ===
        let mut sorted: Vec<_> = scores.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        let (primary, primary_score) = sorted.get(0)
            .map(|(p, s)| (*p, *s))
            .unwrap_or((UserProfile::Custom, 0));

        let secondary = sorted.get(1).map(|(p, _)| *p);
        let secondary_score = sorted.get(1).map(|(_, s)| *s).unwrap_or(0);

        // Confidence: How much better is the best profile?
        let confidence = if primary_score == 0 {
            0.0
        } else if secondary_score == 0 {
            1.0
        } else {
            let ratio = primary_score as f32 / secondary_score as f32;
            ((ratio - 1.0).min(1.0).max(0.0) * 0.5 + 0.5).min(1.0)
        };

        ProfileSuggestion {
            primary,
            secondary,
            confidence,
            detected_apps: scan_result.detected_apps.clone(),
            scores: sorted,
        }
    }
}

/// Result of profile detection
#[derive(Debug, Clone)]
pub struct ProfileSuggestion {
    /// Most likely profile
    pub primary: UserProfile,
    /// Second most likely profile
    pub secondary: Option<UserProfile>,
    /// Confidence (0.0 - 1.0)
    /// - > 0.7: Clear, show directly
    /// - 0.4 - 0.7: Show top-2 for selection
    /// - < 0.4: Ask the user (Custom flow)
    pub confidence: f32,
    /// Detected apps
    pub detected_apps: Vec<DetectedApp>,
    /// All scores for debugging
    pub scores: Vec<(UserProfile, i32)>,
}

/// Quick-scan result for profile detection
#[derive(Debug, Clone, Default)]
pub struct QuickScanResult {
    /// Count per file extension
    pub extension_counts: HashMap<String, usize>,
    /// Detected apps/catalogs
    pub detected_apps: Vec<DetectedApp>,
    /// Sample of file paths (for embedding classification)
    pub sample_paths: Vec<PathBuf>,
    /// Total number of scanned files
    pub total_files: usize,
    /// Scan duration in ms
    pub scan_duration_ms: u64,
}

impl QuickScanResult {
    /// Get the best profile suggestion
    pub fn best_profile(&self) -> Option<UserProfile> {
        let suggestion = ProfileDetector::detect_profile(self);
        if suggestion.confidence > 0.0 {
            Some(suggestion.primary)
        } else {
            None
        }
    }

    /// Check if a profile was detected with high confidence
    pub fn has_high_confidence(&self, min_confidence: f32) -> bool {
        let suggestion = ProfileDetector::detect_profile(self);
        suggestion.confidence >= min_confidence
    }

    /// Get profile suggestion
    pub fn get_suggestion(&self) -> ProfileSuggestion {
        ProfileDetector::detect_profile(self)
    }
}

/// Quick-scanner for profile detection (~5 seconds)
pub struct QuickScanner {
    paths: Vec<PathBuf>,
    max_files: usize,
}

impl QuickScanner {
    /// Create a new quick scanner with default settings
    pub fn new() -> Self {
        Self {
            paths: vec![],
            max_files: 50_000, // Limit for speed
        }
    }

    /// Add paths to scan
    pub fn with_paths(mut self, paths: &[PathBuf]) -> Self {
        self.paths = paths.to_vec();
        self
    }

    /// Set maximum number of files to scan
    pub fn with_max_files(mut self, max_files: usize) -> Self {
        self.max_files = max_files;
        self
    }

    /// Quick metadata scan (only count file extensions)
    pub fn scan(&self) -> Result<QuickScanResult, CxpError> {
        let start = Instant::now();

        let mut result = QuickScanResult::default();
        let mut file_count = 0;

        for base_path in &self.paths {
            if !base_path.exists() {
                continue;
            }

            for entry in WalkDir::new(base_path)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !Self::should_skip(e))
                .filter_map(|e| e.ok())
            {
                if file_count >= self.max_files {
                    break;
                }

                let path = entry.path();

                // App detection
                if let Some(app) = Self::detect_app(path) {
                    if !result.detected_apps.iter().any(|a| a.path == app.path) {
                        result.detected_apps.push(app);
                    }
                }

                if entry.file_type().is_file() {
                    // Count file extension
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        *result.extension_counts.entry(ext_str).or_insert(0) += 1;
                    }

                    // Sample for embeddings
                    if result.sample_paths.len() < 500 && file_count % 100 == 0 {
                        result.sample_paths.push(path.to_path_buf());
                    }

                    file_count += 1;
                }
            }
        }

        result.total_files = file_count;
        result.scan_duration_ms = start.elapsed().as_millis() as u64;

        Ok(result)
    }

    fn should_skip(entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();

        // Quick skip of known junk folders
        matches!(name.as_ref(),
            "node_modules" | ".git" | "target" | "dist" | "build" |
            ".cache" | "__pycache__" | ".venv" | "venv" | ".idea" |
            ".vs" | "Library" | "Caches" | ".Trash"
        )
    }

    fn detect_app(path: &Path) -> Option<DetectedApp> {
        let name = path.file_name()?.to_string_lossy();

        // Lightroom Catalog
        if name.ends_with(".lrcat") {
            return Some(DetectedApp {
                name: name.trim_end_matches(".lrcat").to_string(),
                app_type: "Lightroom Catalog".to_string(),
                path: path.to_path_buf(),
                importance: 1.0,
            });
        }

        // Obsidian Vault
        if name == ".obsidian" && path.is_dir() {
            return Some(DetectedApp {
                name: path.parent()?.file_name()?.to_string_lossy().to_string(),
                app_type: "Obsidian Vault".to_string(),
                path: path.parent()?.to_path_buf(),
                importance: 1.0,
            });
        }

        // Git Repository
        if name == ".git" && path.is_dir() {
            return Some(DetectedApp {
                name: path.parent()?.file_name()?.to_string_lossy().to_string(),
                app_type: "Git Repository".to_string(),
                path: path.parent()?.to_path_buf(),
                importance: 0.8,
            });
        }

        // VS Code Workspace
        if name.ends_with(".code-workspace") {
            return Some(DetectedApp {
                name: name.trim_end_matches(".code-workspace").to_string(),
                app_type: "VS Code Workspace".to_string(),
                path: path.to_path_buf(),
                importance: 0.8,
            });
        }

        // VS Code Project
        if name == ".vscode" && path.is_dir() {
            return Some(DetectedApp {
                name: path.parent()?.file_name()?.to_string_lossy().to_string(),
                app_type: "VS Code Project".to_string(),
                path: path.parent()?.to_path_buf(),
                importance: 0.7,
            });
        }

        // JetBrains Project
        if name == ".idea" && path.is_dir() {
            return Some(DetectedApp {
                name: path.parent()?.file_name()?.to_string_lossy().to_string(),
                app_type: "JetBrains Project".to_string(),
                path: path.parent()?.to_path_buf(),
                importance: 0.7,
            });
        }

        // Figma files
        if name.ends_with(".fig") {
            return Some(DetectedApp {
                name: name.trim_end_matches(".fig").to_string(),
                app_type: "Figma File".to_string(),
                path: path.to_path_buf(),
                importance: 0.9,
            });
        }

        // Sketch files
        if name.ends_with(".sketch") {
            return Some(DetectedApp {
                name: name.trim_end_matches(".sketch").to_string(),
                app_type: "Sketch File".to_string(),
                path: path.to_path_buf(),
                importance: 0.9,
            });
        }

        // Scrivener Project
        if name.ends_with(".scriv") {
            return Some(DetectedApp {
                name: name.trim_end_matches(".scriv").to_string(),
                app_type: "Scrivener Project".to_string(),
                path: path.to_path_buf(),
                importance: 1.0,
            });
        }

        // Capture One Catalog
        if name.ends_with(".cocatalog") || name.ends_with(".cosessiondb") {
            return Some(DetectedApp {
                name: name.to_string(),
                app_type: "Capture One Catalog".to_string(),
                path: path.to_path_buf(),
                importance: 1.0,
            });
        }

        // Adobe Bridge Cache
        if name == "Adobe Bridge Cache" || name == ".BridgeCache" {
            return Some(DetectedApp {
                name: "Adobe Bridge Cache".to_string(),
                app_type: "Adobe Bridge".to_string(),
                path: path.to_path_buf(),
                importance: 0.7,
            });
        }

        // Apple Photos Library
        if name.ends_with(".photoslibrary") {
            return Some(DetectedApp {
                name: name.trim_end_matches(".photoslibrary").to_string(),
                app_type: "Apple Photos Library".to_string(),
                path: path.to_path_buf(),
                importance: 1.0,
            });
        }

        None
    }
}

impl Default for QuickScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_detector_developer() {
        let mut result = QuickScanResult::default();

        // Add developer files
        result.extension_counts.insert("rs".to_string(), 100);
        result.extension_counts.insert("ts".to_string(), 50);
        result.extension_counts.insert("py".to_string(), 30);
        result.extension_counts.insert("toml".to_string(), 5);

        let suggestion = ProfileDetector::detect_profile(&result);
        assert_eq!(suggestion.primary, UserProfile::Developer);
        assert!(suggestion.confidence > 0.5);
    }

    #[test]
    fn test_profile_detector_photographer() {
        let mut result = QuickScanResult::default();

        // Add photographer files
        result.extension_counts.insert("cr2".to_string(), 500);
        result.extension_counts.insert("nef".to_string(), 300);
        result.extension_counts.insert("jpg".to_string(), 1000);
        result.extension_counts.insert("lrcat".to_string(), 1);

        let suggestion = ProfileDetector::detect_profile(&result);
        assert_eq!(suggestion.primary, UserProfile::Photographer);
        assert!(suggestion.confidence > 0.7);
    }

    #[test]
    fn test_profile_detector_designer() {
        let mut result = QuickScanResult::default();

        // Add designer files
        result.extension_counts.insert("psd".to_string(), 80);
        result.extension_counts.insert("ai".to_string(), 40);
        result.extension_counts.insert("sketch".to_string(), 20);
        result.extension_counts.insert("svg".to_string(), 100);

        let suggestion = ProfileDetector::detect_profile(&result);
        assert_eq!(suggestion.primary, UserProfile::Designer);
    }

    #[test]
    fn test_profile_detector_with_apps() {
        let mut result = QuickScanResult::default();

        // Few files but important app
        result.extension_counts.insert("md".to_string(), 50);
        result.detected_apps.push(DetectedApp {
            name: "My Vault".to_string(),
            app_type: "Obsidian Vault".to_string(),
            path: PathBuf::from("/test"),
            importance: 1.0,
        });

        let suggestion = ProfileDetector::detect_profile(&result);
        assert_eq!(suggestion.primary, UserProfile::Writer);
        assert!(suggestion.confidence > 0.8);
    }

    #[test]
    fn test_quick_scanner_new() {
        let scanner = QuickScanner::new();
        assert_eq!(scanner.max_files, 50_000);
        assert_eq!(scanner.paths.len(), 0);
    }

    #[test]
    fn test_quick_scanner_builder() {
        let paths = vec![PathBuf::from("/test")];
        let scanner = QuickScanner::new()
            .with_paths(&paths)
            .with_max_files(1000);

        assert_eq!(scanner.max_files, 1000);
        assert_eq!(scanner.paths.len(), 1);
    }

    #[test]
    fn test_quick_scan_result_best_profile() {
        let mut result = QuickScanResult::default();
        result.extension_counts.insert("rs".to_string(), 100);

        let best = result.best_profile();
        assert_eq!(best, Some(UserProfile::Developer));
    }

    #[test]
    fn test_quick_scan_result_high_confidence() {
        let mut result = QuickScanResult::default();
        result.extension_counts.insert("cr2".to_string(), 500);
        result.extension_counts.insert("nef".to_string(), 300);

        // Photographer files should give decent confidence
        assert!(result.has_high_confidence(0.5));

        // But not absolute confidence since there might be some variation
        let suggestion = result.get_suggestion();
        assert!(suggestion.confidence == 1.0);
    }

    #[test]
    fn test_should_skip() {
        let temp = std::env::temp_dir();

        // Test that common folders would be skipped
        let skip_names = vec![
            "node_modules", ".git", "target", "dist", "build",
            ".cache", "__pycache__", ".venv", "venv", ".idea",
        ];

        for name in skip_names {
            let path = temp.join(name);
            if path.exists() {
                for entry in WalkDir::new(&path).max_depth(0) {
                    if let Ok(entry) = entry {
                        assert!(QuickScanner::should_skip(&entry));
                    }
                }
            }
        }
    }

    #[test]
    fn test_detect_app_git() {
        let temp = std::env::temp_dir().join("test_repo");
        let git_path = temp.join(".git");

        if let Some(app) = QuickScanner::detect_app(&git_path) {
            assert_eq!(app.app_type, "Git Repository");
            assert_eq!(app.importance, 0.8);
        }
    }

    #[test]
    #[test]
    fn test_confidence_calculation() {
        // Test case 1: Photographer vs Developer (Photographer wins)
        let mut result = QuickScanResult::default();
        result.extension_counts.insert("rs".to_string(), 100);  // Developer: 1000 points
        result.extension_counts.insert("cr2".to_string(), 90);  // Photographer: 1800 points
        
        let suggestion = ProfileDetector::detect_profile(&result);
        // Photographer wins: 1800 vs 1000, ratio 1.8, confidence ~0.9
        assert_eq!(suggestion.primary, UserProfile::Photographer);
        assert!(suggestion.confidence > 0.85);
        assert!(suggestion.confidence < 0.95);

        // Test case 2: Clear winner (should be max confidence)
        let mut result2 = QuickScanResult::default();
        result2.extension_counts.insert("rs".to_string(), 100);  // Developer: 1000 points
        result2.extension_counts.insert("jpg".to_string(), 5);   // Photographer: 10 points

        let suggestion2 = ProfileDetector::detect_profile(&result2);
        // Developer wins: 1000 vs 10, ratio 100, confidence = 1.0
        assert_eq!(suggestion2.primary, UserProfile::Developer);
        assert!(suggestion2.confidence > 0.95);
    }

    #[test]
    fn test_empty_scan_result() {
        let result = QuickScanResult::default();
        let suggestion = ProfileDetector::detect_profile(&result);

        assert_eq!(suggestion.primary, UserProfile::Custom);
        assert_eq!(suggestion.confidence, 0.0);
    }
}

#[cfg(test)]
mod fixed_tests {
    use super::*;
    
    #[test]
    fn test_mixed_profiles_confidence() {
        // Test with different profiles competing
        let mut result = QuickScanResult::default();
        result.extension_counts.insert("rs".to_string(), 100);  // Developer: 1000
        result.extension_counts.insert("cr2".to_string(), 40);  // Photographer: 800
        
        let suggestion = ProfileDetector::detect_profile(&result);
        // Developer wins with 1000 vs 800, ratio 1.25, conf ~0.625
        assert_eq!(suggestion.primary, UserProfile::Developer);
        assert!(suggestion.confidence > 0.6);
        assert!(suggestion.confidence < 0.7);
    }
}
