use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use super::config::ScanConfig;

/// User profiles for intelligent scan configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserProfile {
    /// Developer - Code, Configs, Documentation
    Developer,
    /// Photographer - RAW, JPEG, Lightroom, Catalogs
    Photographer,
    /// Designer - Figma, Sketch, Adobe, Assets
    Designer,
    /// Writer - Markdown, Word, Notes, Research
    Writer,
    /// Student - Mix of everything
    Student,
    /// Business - Excel, Reports, Presentations
    Business,
    /// Custom - User chooses everything
    Custom,
}

impl Default for UserProfile {
    fn default() -> Self {
        UserProfile::Developer
    }
}

impl UserProfile {
    /// Returns the default ScanConfig for this profile
    pub fn default_config(&self) -> ScanConfig {
        match self {
            UserProfile::Developer => ScanConfig {
                paths: vec![],
                file_extensions: vec![
                    // Code
                    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "cpp", "c", "h",
                    "rb", "php", "swift", "kt", "scala", "clj", "ex", "exs", "erl",
                    // Config
                    "json", "yaml", "yml", "toml", "xml", "env", "ini", "conf",
                    // Docs
                    "md", "txt", "pdf", "rst", "adoc",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                max_file_size: 10 * 1024 * 1024, // 10 MB
                include_images: false,           // Only screenshots
                include_hidden: false,
                custom_ignore: vec![],
                force_include: vec![],
            },

            UserProfile::Photographer => ScanConfig {
                paths: vec![],
                file_extensions: vec![
                    // RAW formats
                    "raw", "cr2", "cr3", "nef", "arw", "orf", "dng", "rw2", "raf", "srw",
                    "pef", "x3f", "erf", "mrw", "dcr", "kdc",
                    // Standard
                    "jpg", "jpeg", "png", "tiff", "tif", "heic", "heif", "webp",
                    // Lightroom
                    "lrcat", "lrtemplate", "xmp",
                    // Video
                    "mp4", "mov", "avi", "mkv", "mts", "m2ts",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                max_file_size: 100 * 1024 * 1024, // 100 MB (RAW files)
                include_images: true,
                include_hidden: false,
                custom_ignore: vec![],
                force_include: vec![],
            },

            UserProfile::Designer => ScanConfig {
                paths: vec![],
                file_extensions: vec![
                    // Design Tools
                    "fig",      // Figma (local)
                    "sketch",   // Sketch
                    "xd",       // Adobe XD
                    "psd",      // Photoshop
                    "ai",       // Illustrator
                    "indd",     // InDesign
                    "afdesign", // Affinity Designer
                    "afphoto",  // Affinity Photo
                    // Export
                    "svg", "png", "jpg", "jpeg", "pdf", "eps",
                    // Prototyping
                    "framerx", "principle",
                    // Specs
                    "json", "yaml", // Design tokens
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                max_file_size: 200 * 1024 * 1024, // 200 MB (PSD files)
                include_images: true,
                include_hidden: false,
                custom_ignore: vec![],
                force_include: vec![],
            },

            UserProfile::Writer => ScanConfig {
                paths: vec![],
                file_extensions: vec![
                    // Text
                    "md", "txt", "rtf",
                    // Office
                    "docx", "doc", "odt", "pages",
                    // Notes
                    "org", "tex", "latex",
                    // Research
                    "pdf", "epub", "mobi",
                    // Obsidian/Notes
                    "canvas",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                max_file_size: 50 * 1024 * 1024, // 50 MB
                include_images: false,
                include_hidden: false,
                custom_ignore: vec![],
                force_include: vec![],
            },

            UserProfile::Student => ScanConfig {
                paths: vec![],
                file_extensions: vec![
                    // Docs
                    "md", "txt", "pdf", "docx", "pptx", "xlsx", "odt", "odp", "ods",
                    // Code (courses)
                    "py", "java", "js", "c", "cpp", "h", "cs", "r",
                    // Notes
                    "org", "tex",
                    // Media
                    "mp4", "mp3", "m4a", "webm",
                    // Images
                    "png", "jpg", "jpeg",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                max_file_size: 500 * 1024 * 1024, // 500 MB (Lectures)
                include_images: true,              // Screenshots from lectures
                include_hidden: false,
                custom_ignore: vec![],
                force_include: vec![],
            },

            UserProfile::Business => ScanConfig {
                paths: vec![],
                file_extensions: vec![
                    // Office
                    "docx", "doc", "xlsx", "xls", "pptx", "ppt", "pdf",
                    // Data
                    "csv", "tsv",
                    // Email
                    "eml", "msg",
                    // Other
                    "odt", "ods", "odp",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                max_file_size: 50 * 1024 * 1024,
                include_images: false,
                include_hidden: false,
                custom_ignore: vec![],
                force_include: vec![],
            },

            UserProfile::Custom => ScanConfig::default(),
        }
    }

    /// Special app/catalog detectors per profile
    pub fn special_detectors(&self) -> Vec<SpecialDetector> {
        match self {
            UserProfile::Photographer => vec![
                SpecialDetector::Lightroom,
                SpecialDetector::CaptureOne,
                SpecialDetector::AdobeBridge,
                SpecialDetector::ApplePhotos,
            ],
            UserProfile::Designer => vec![
                SpecialDetector::FigmaLocal,
                SpecialDetector::Sketch,
                SpecialDetector::AdobeCreativeCloud,
            ],
            UserProfile::Writer => vec![
                SpecialDetector::Obsidian,
                SpecialDetector::Notion,
                SpecialDetector::Scrivener,
            ],
            UserProfile::Developer => vec![
                SpecialDetector::GitRepo,
                SpecialDetector::VSCodeWorkspace,
                SpecialDetector::JetBrainsProject,
            ],
            _ => vec![],
        }
    }

    /// Get all available profiles
    pub fn all() -> Vec<UserProfile> {
        vec![
            UserProfile::Developer,
            UserProfile::Photographer,
            UserProfile::Designer,
            UserProfile::Writer,
            UserProfile::Student,
            UserProfile::Business,
            UserProfile::Custom,
        ]
    }

    /// Get a human-readable name for the profile
    pub fn name(&self) -> &'static str {
        match self {
            UserProfile::Developer => "Developer",
            UserProfile::Photographer => "Photographer",
            UserProfile::Designer => "Designer",
            UserProfile::Writer => "Writer",
            UserProfile::Student => "Student",
            UserProfile::Business => "Business",
            UserProfile::Custom => "Custom",
        }
    }

    /// Get a description of the profile
    pub fn description(&self) -> &'static str {
        match self {
            UserProfile::Developer => "Software development with code, configs, and documentation",
            UserProfile::Photographer => "Photo editing and management with RAW files and exports",
            UserProfile::Designer => "Design work with mockups, assets, and design files",
            UserProfile::Writer => "Writing and documentation with text files and drafts",
            UserProfile::Student => "Academic work with lectures, notes, and assignments",
            UserProfile::Business => "Business documents, reports, and presentations",
            UserProfile::Custom => "Custom configuration based on your specific needs",
        }
    }

    /// Get special folders to prioritize for this profile
    pub fn special_folders(&self) -> Vec<&'static str> {
        match self {
            UserProfile::Developer => vec!["src", "lib", "app", "docs", "config"],
            UserProfile::Photographer => vec!["Lightroom", "Photos", "Pictures", "Camera Roll", "DCIM"],
            UserProfile::Designer => vec!["Design", "Assets", "Mockups", "UI", "UX", "Prototypes"],
            UserProfile::Writer => vec!["Documents", "Notes", "Writing", "Research", "Obsidian", "Notion", "Drafts"],
            UserProfile::Student => vec!["University", "Uni", "School", "Courses", "Semester", "Lectures"],
            UserProfile::Business => vec!["Documents", "Reports", "Presentations", "Contracts", "Invoices"],
            UserProfile::Custom => vec![],
        }
    }
}

/// Special app/catalog detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialDetector {
    // Photographer
    Lightroom,
    CaptureOne,
    AdobeBridge,
    ApplePhotos,

    // Designer
    FigmaLocal,
    Sketch,
    AdobeCreativeCloud,

    // Writer
    Obsidian,
    Notion,
    Scrivener,

    // Developer
    GitRepo,
    VSCodeWorkspace,
    JetBrainsProject,
}

impl SpecialDetector {
    /// Detects if this special folder/app exists at the given path
    pub fn detect(&self, path: &Path) -> Option<DetectedApp> {
        match self {
            SpecialDetector::Lightroom => {
                // Search for .lrcat files
                if path.extension().map(|e| e == "lrcat").unwrap_or(false) {
                    return Some(DetectedApp {
                        name: path
                            .file_name()?
                            .to_string_lossy()
                            .trim_end_matches(".lrcat")
                            .to_string(),
                        app_type: "Lightroom Catalog".to_string(),
                        path: path.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            }

            SpecialDetector::CaptureOne => {
                // Search for .cocatalog or .cosessiondb files
                let ext = path.extension()?.to_string_lossy();
                if ext == "cocatalog" || ext == "cosessiondb" {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Capture One Catalog".to_string(),
                        path: path.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            }

            SpecialDetector::AdobeBridge => {
                // Search for Adobe Bridge Cache folders
                let name = path.file_name()?.to_string_lossy();
                if name == "Adobe Bridge Cache" || name == ".BridgeCache" {
                    return Some(DetectedApp {
                        name: "Adobe Bridge Cache".to_string(),
                        app_type: "Adobe Bridge".to_string(),
                        path: path.to_path_buf(),
                        importance: 0.7,
                    });
                }
                None
            }

            SpecialDetector::ApplePhotos => {
                // Search for .photoslibrary packages
                if path
                    .extension()
                    .map(|e| e == "photoslibrary")
                    .unwrap_or(false)
                {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Apple Photos Library".to_string(),
                        path: path.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            }

            SpecialDetector::FigmaLocal => {
                // Search for .fig files
                if path.extension().map(|e| e == "fig").unwrap_or(false) {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Figma File".to_string(),
                        path: path.to_path_buf(),
                        importance: 0.9,
                    });
                }
                None
            }

            SpecialDetector::Sketch => {
                // Search for .sketch files
                if path.extension().map(|e| e == "sketch").unwrap_or(false) {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Sketch File".to_string(),
                        path: path.to_path_buf(),
                        importance: 0.9,
                    });
                }
                None
            }

            SpecialDetector::AdobeCreativeCloud => {
                // Search for Adobe CC folders
                let name = path.file_name()?.to_string_lossy();
                if name == "Adobe" || name == "Creative Cloud Files" {
                    return Some(DetectedApp {
                        name: name.to_string(),
                        app_type: "Adobe Creative Cloud".to_string(),
                        path: path.to_path_buf(),
                        importance: 0.8,
                    });
                }
                None
            }

            SpecialDetector::Obsidian => {
                // Search for .obsidian folder
                let name = path.file_name()?.to_string_lossy();
                if name == ".obsidian" && path.is_dir() {
                    return Some(DetectedApp {
                        name: path
                            .parent()?
                            .file_name()?
                            .to_string_lossy()
                            .to_string(),
                        app_type: "Obsidian Vault".to_string(),
                        path: path.parent()?.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            }

            SpecialDetector::Notion => {
                // Notion is usually cloud-based, but check for export folders
                let name = path.file_name()?.to_string_lossy();
                if name.contains("Notion") && name.contains("Export") {
                    return Some(DetectedApp {
                        name: name.to_string(),
                        app_type: "Notion Export".to_string(),
                        path: path.to_path_buf(),
                        importance: 0.8,
                    });
                }
                None
            }

            SpecialDetector::Scrivener => {
                // Search for .scriv packages
                if path.extension().map(|e| e == "scriv").unwrap_or(false) {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Scrivener Project".to_string(),
                        path: path.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            }

            SpecialDetector::GitRepo => {
                // Search for .git folder
                let name = path.file_name()?.to_string_lossy();
                if name == ".git" && path.is_dir() {
                    return Some(DetectedApp {
                        name: path
                            .parent()?
                            .file_name()?
                            .to_string_lossy()
                            .to_string(),
                        app_type: "Git Repository".to_string(),
                        path: path.parent()?.to_path_buf(),
                        importance: 0.9,
                    });
                }
                None
            }

            SpecialDetector::VSCodeWorkspace => {
                // Search for .code-workspace files or .vscode folder
                if path
                    .extension()
                    .map(|e| e == "code-workspace")
                    .unwrap_or(false)
                {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "VS Code Workspace".to_string(),
                        path: path.to_path_buf(),
                        importance: 0.8,
                    });
                }

                let name = path.file_name()?.to_string_lossy();
                if name == ".vscode" && path.is_dir() {
                    return Some(DetectedApp {
                        name: path
                            .parent()?
                            .file_name()?
                            .to_string_lossy()
                            .to_string(),
                        app_type: "VS Code Project".to_string(),
                        path: path.parent()?.to_path_buf(),
                        importance: 0.7,
                    });
                }
                None
            }

            SpecialDetector::JetBrainsProject => {
                // Search for .idea folder
                let name = path.file_name()?.to_string_lossy();
                if name == ".idea" && path.is_dir() {
                    return Some(DetectedApp {
                        name: path
                            .parent()?
                            .file_name()?
                            .to_string_lossy()
                            .to_string(),
                        app_type: "JetBrains Project".to_string(),
                        path: path.parent()?.to_path_buf(),
                        importance: 0.7,
                    });
                }
                None
            }
        }
    }
}

/// Detected application/catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedApp {
    /// Name of the detected app/project
    pub name: String,
    /// Type of app (e.g., "Lightroom Catalog", "Git Repository")
    pub app_type: String,
    /// Path to the app/project
    pub path: PathBuf,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_developer_profile_config() {
        let profile = UserProfile::Developer;
        let config = profile.default_config();

        assert!(!config.include_images);

        // Check for common code extensions
        assert!(config.file_extensions.contains(&"rs".to_string()));
        assert!(config.file_extensions.contains(&"ts".to_string()));
        assert!(config.file_extensions.contains(&"py".to_string()));
    }

    #[test]
    fn test_photographer_profile_config() {
        let profile = UserProfile::Photographer;
        let config = profile.default_config();

        assert!(config.include_images);

        // Check for RAW formats
        assert!(config.file_extensions.contains(&"cr2".to_string()));
        assert!(config.file_extensions.contains(&"nef".to_string()));
        assert!(config.file_extensions.contains(&"lrcat".to_string()));

        // Higher max file size for RAW files
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_designer_profile_config() {
        let profile = UserProfile::Designer;
        let config = profile.default_config();

        assert!(config.include_images);

        // Check for design tool formats
        assert!(config.file_extensions.contains(&"psd".to_string()));
        assert!(config.file_extensions.contains(&"ai".to_string()));
        assert!(config.file_extensions.contains(&"sketch".to_string()));
    }

    #[test]
    fn test_special_detectors() {
        let dev_profile = UserProfile::Developer;
        let detectors = dev_profile.special_detectors();

        assert!(detectors.contains(&SpecialDetector::GitRepo));
        assert!(detectors.contains(&SpecialDetector::VSCodeWorkspace));
        assert!(detectors.contains(&SpecialDetector::JetBrainsProject));

        let photo_profile = UserProfile::Photographer;
        let detectors = photo_profile.special_detectors();

        assert!(detectors.contains(&SpecialDetector::Lightroom));
        assert!(detectors.contains(&SpecialDetector::CaptureOne));
    }

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert_eq!(profile, UserProfile::Developer);
    }

    #[test]
    fn test_custom_profile() {
        let profile = UserProfile::Custom;
        let config = profile.default_config();

        // Custom profile should use default ScanConfig
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_all_profiles() {
        let profiles = UserProfile::all();
        assert_eq!(profiles.len(), 7);
        assert!(profiles.contains(&UserProfile::Developer));
        assert!(profiles.contains(&UserProfile::Student));
        assert!(profiles.contains(&UserProfile::Business));
    }

    #[test]
    fn test_profile_names() {
        assert_eq!(UserProfile::Developer.name(), "Developer");
        assert_eq!(UserProfile::Photographer.name(), "Photographer");
        assert_eq!(UserProfile::Student.name(), "Student");
        assert_eq!(UserProfile::Business.name(), "Business");
    }
}
