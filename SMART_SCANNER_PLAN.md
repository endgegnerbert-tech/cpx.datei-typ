# Smart Scanner Plan für CXP

## Ziel

Ein intelligentes Scanning-System das:
1. **Automatisch** irrelevante Dateien ignoriert
2. **KI-gestützt** den Nutzer bei Entscheidungen unterstützt (< $0.10 Kosten)
3. **Kategorisiert** nach Typ (Code, Docs, Images)
4. **Priorisiert** nach Aktivität (HOT/WARM/COLD)
5. **Skaliert** von 1 GB bis 1 TB ohne Probleme
6. **Personalisiert** nach Nutzer-Profil (Developer, Photographer, Designer, etc.)

---

## User Profile System

### Warum Profile?

Nicht jeder Nutzer ist gleich:
- **Developer** → Code, Configs, Docs sind wichtig
- **Photographer** → RAW-Fotos, Lightroom-Kataloge, EXIF-Daten
- **Designer** → Figma, Sketch, Adobe-Dateien, Assets
- **Writer** → Markdown, Word, Notizen, Research
- **Student** → Mix aus allem, Vorlesungen, PDFs
- **Business** → Excel, Reports, E-Mails, Präsentationen

### Profile Enum

```rust
// scanner/profile.rs

/// Nutzer-Profile für intelligente Scan-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserProfile {
    /// Entwickler - Code, Configs, Dokumentation
    Developer,
    /// Fotograf - RAW, JPEG, Lightroom, Kataloge
    Photographer,
    /// Designer - Figma, Sketch, Adobe, Assets
    Designer,
    /// Autor - Markdown, Word, Notizen, Research
    Writer,
    /// Student - Mix aus allem
    Student,
    /// Business - Excel, Reports, Präsentationen
    Business,
    /// Benutzerdefiniert - Nutzer wählt alles selbst
    Custom,
}

impl Default for UserProfile {
    fn default() -> Self {
        UserProfile::Developer
    }
}
```

### Profile-spezifische Konfigurationen

```rust
impl UserProfile {
    /// Gibt die Standard-ScanConfig für dieses Profil zurück
    pub fn default_config(&self) -> ScanConfig {
        match self {
            UserProfile::Developer => ScanConfig {
                include_code: true,
                include_configs: true,
                include_docs: true,
                include_images: false,  // Nur Screenshots
                include_media: false,
                file_extensions: vec![
                    // Code
                    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "cpp", "c", "h",
                    // Config
                    "json", "yaml", "yml", "toml", "xml", "env",
                    // Docs
                    "md", "txt", "pdf",
                ],
                max_file_size: 10 * 1024 * 1024,  // 10 MB
                ..Default::default()
            },

            UserProfile::Photographer => ScanConfig {
                include_code: false,
                include_configs: false,
                include_docs: false,
                include_images: true,
                include_media: true,
                file_extensions: vec![
                    // RAW Formate
                    "raw", "cr2", "cr3", "nef", "arw", "orf", "dng", "rw2",
                    // Standard
                    "jpg", "jpeg", "png", "tiff", "tif", "heic", "heif",
                    // Lightroom
                    "lrcat", "lrtemplate", "xmp",
                    // Video
                    "mp4", "mov", "avi",
                ],
                max_file_size: 100 * 1024 * 1024,  // 100 MB (RAW files)
                special_folders: vec![
                    "Lightroom",
                    "Photos",
                    "Pictures",
                    "Camera Roll",
                ],
                ..Default::default()
            },

            UserProfile::Designer => ScanConfig {
                include_code: false,
                include_configs: true,  // Design system tokens
                include_docs: true,
                include_images: true,
                include_media: false,
                file_extensions: vec![
                    // Design Tools
                    "fig",      // Figma (lokal)
                    "sketch",   // Sketch
                    "xd",       // Adobe XD
                    "psd",      // Photoshop
                    "ai",       // Illustrator
                    "indd",     // InDesign
                    // Export
                    "svg", "png", "jpg", "pdf",
                    // Prototyping
                    "framerx", "principle",
                    // Specs
                    "json", "yaml",  // Design tokens
                ],
                max_file_size: 200 * 1024 * 1024,  // 200 MB (PSD files)
                special_folders: vec![
                    "Design",
                    "Assets",
                    "Mockups",
                    "UI",
                    "UX",
                ],
                ..Default::default()
            },

            UserProfile::Writer => ScanConfig {
                include_code: false,
                include_configs: false,
                include_docs: true,
                include_images: false,
                include_media: false,
                file_extensions: vec![
                    // Text
                    "md", "txt", "rtf",
                    // Office
                    "docx", "doc", "odt",
                    "pages",
                    // Notes
                    "org", "tex", "latex",
                    // Research
                    "pdf", "epub",
                    // Obsidian/Notes
                    "canvas",
                ],
                max_file_size: 50 * 1024 * 1024,  // 50 MB
                special_folders: vec![
                    "Documents",
                    "Notes",
                    "Writing",
                    "Research",
                    "Obsidian",
                    "Notion",
                ],
                ..Default::default()
            },

            UserProfile::Student => ScanConfig {
                include_code: true,  // Programmier-Kurse
                include_configs: false,
                include_docs: true,
                include_images: true,  // Screenshots von Vorlesungen
                include_media: true,   // Vorlesungsaufnahmen
                file_extensions: vec![
                    // Docs
                    "md", "txt", "pdf", "docx", "pptx", "xlsx",
                    // Code (Kurse)
                    "py", "java", "js", "c", "cpp",
                    // Notes
                    "org",
                    // Media
                    "mp4", "mp3", "m4a",
                ],
                max_file_size: 500 * 1024 * 1024,  // 500 MB (Vorlesungen)
                special_folders: vec![
                    "University",
                    "Uni",
                    "School",
                    "Courses",
                    "Semester",
                ],
                ..Default::default()
            },

            UserProfile::Business => ScanConfig {
                include_code: false,
                include_configs: false,
                include_docs: true,
                include_images: false,
                include_media: false,
                file_extensions: vec![
                    // Office
                    "docx", "doc", "xlsx", "xls", "pptx", "ppt",
                    "pdf",
                    // Data
                    "csv",
                    // Email
                    "eml", "msg",
                ],
                max_file_size: 50 * 1024 * 1024,
                special_folders: vec![
                    "Documents",
                    "Reports",
                    "Presentations",
                    "Contracts",
                ],
                ..Default::default()
            },

            UserProfile::Custom => ScanConfig::default(),
        }
    }

    /// Spezielle App/Katalog-Erkennung pro Profil
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
}

/// Spezielle App/Katalog-Erkennung
#[derive(Debug, Clone)]
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
    /// Erkennt ob dieser spezielle Ordner/App existiert
    pub fn detect(&self, path: &Path) -> Option<DetectedApp> {
        match self {
            SpecialDetector::Lightroom => {
                // Suche nach .lrcat Dateien
                if path.extension().map(|e| e == "lrcat").unwrap_or(false) {
                    return Some(DetectedApp {
                        name: "Lightroom Catalog".to_string(),
                        app_type: "Lightroom",
                        path: path.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            },
            SpecialDetector::Obsidian => {
                // Suche nach .obsidian Ordner
                if path.join(".obsidian").exists() {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Obsidian Vault",
                        path: path.to_path_buf(),
                        importance: 1.0,
                    });
                }
                None
            },
            SpecialDetector::GitRepo => {
                if path.join(".git").exists() {
                    return Some(DetectedApp {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        app_type: "Git Repository",
                        path: path.to_path_buf(),
                        importance: 0.9,
                    });
                }
                None
            },
            // ... weitere Detektoren
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectedApp {
    pub name: String,
    pub app_type: &'static str,
    pub path: PathBuf,
    pub importance: f32,
}
```

### Kostenlose Profil-Erkennung (100% Lokal)

```rust
// scanner/profile_detector.rs

use std::collections::HashMap;

/// Profil-Erkennung: Komplett kostenlos & lokal
///
/// Strategie:
/// 1. Regelbasiert: Zähle Dateitypen (90% der Fälle)
/// 2. App-Erkennung: Lightroom, Obsidian, VS Code, etc.
/// 3. Embedding-Klassifikation: Nutze vorhandenes MiniLM-Modell (Optional)
///
/// Keine API-Kosten! Keine Internetverbindung nötig!
pub struct ProfileDetector;

impl ProfileDetector {
    /// Analysiert den PC und schlägt ein Profil vor
    pub fn detect_profile(scan_result: &QuickScanResult) -> ProfileSuggestion {
        let mut scores: HashMap<UserProfile, i32> = HashMap::new();

        // === SCHRITT 1: Dateitypen zählen ===
        for (ext, count) in &scan_result.extension_counts {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                // Developer (hohe Gewichtung)
                "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" |
                "cpp" | "c" | "h" | "rb" | "php" | "swift" | "kt" => {
                    *scores.entry(UserProfile::Developer).or_insert(0) += (*count as i32) * 10;
                },
                // Photographer (sehr hohe Gewichtung für RAW)
                "raw" | "cr2" | "cr3" | "nef" | "arw" | "orf" | "dng" | "rw2" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += (*count as i32) * 20;
                },
                "jpg" | "jpeg" | "heic" | "heif" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += (*count as i32) * 2;
                },
                "lrcat" | "lrtemplate" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += 500; // Lightroom = Fotograf
                },
                // Designer
                "fig" | "sketch" | "xd" | "psd" | "ai" | "indd" => {
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
                    // PDF ist universal - kleine Gewichtung für alle
                    *scores.entry(UserProfile::Writer).or_insert(0) += (*count as i32) * 1;
                    *scores.entry(UserProfile::Business).or_insert(0) += (*count as i32) * 1;
                    *scores.entry(UserProfile::Student).or_insert(0) += (*count as i32) * 1;
                },
                _ => {},
            }
        }

        // === SCHRITT 2: Erkannte Apps einbeziehen ===
        for app in &scan_result.detected_apps {
            match app.app_type {
                "Lightroom" | "CaptureOne" | "AdobeBridge" => {
                    *scores.entry(UserProfile::Photographer).or_insert(0) += 1000;
                },
                "Figma" | "Sketch" | "AdobeCC" => {
                    *scores.entry(UserProfile::Designer).or_insert(0) += 1000;
                },
                "Obsidian" | "Notion" | "Scrivener" => {
                    *scores.entry(UserProfile::Writer).or_insert(0) += 1000;
                },
                "Git Repository" | "VSCode" | "JetBrains" => {
                    *scores.entry(UserProfile::Developer).or_insert(0) += 500;
                },
                _ => {},
            }
        }

        // === SCHRITT 3: Confidence berechnen ===
        let mut sorted: Vec<_> = scores.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        let (primary, primary_score) = sorted.get(0)
            .map(|(p, s)| (p.clone(), *s))
            .unwrap_or((UserProfile::Custom, 0));

        let secondary = sorted.get(1).map(|(p, _)| p.clone());
        let secondary_score = sorted.get(1).map(|(_, s)| *s).unwrap_or(0);

        // Confidence: Wie viel besser ist das beste Profil?
        let confidence = if primary_score == 0 {
            0.0
        } else if secondary_score == 0 {
            1.0
        } else {
            let ratio = primary_score as f32 / secondary_score as f32;
            (ratio - 1.0).min(1.0).max(0.0) * 0.5 + 0.5
        };

        ProfileSuggestion {
            primary,
            secondary,
            confidence,
            detected_apps: scan_result.detected_apps.clone(),
            scores: sorted,
        }
    }

    /// Optional: Embedding-basierte Klassifikation für unsichere Fälle
    /// Nutzt das bereits geladene MiniLM-Modell - KEINE API-KOSTEN!
    #[cfg(feature = "embeddings")]
    pub fn classify_with_embeddings(
        scan_result: &QuickScanResult,
        embedding_engine: &EmbeddingEngine,
    ) -> Option<UserProfile> {
        // Erstelle einen beschreibenden Text aus den Dateipfaden
        let description = scan_result.sample_paths
            .iter()
            .take(100)
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        // Profil-Prototypen (vorberechnet)
        let prototypes = [
            (UserProfile::Developer, "source code typescript python rust git repository package.json cargo.toml"),
            (UserProfile::Photographer, "photos raw cr2 nef lightroom catalog jpeg pictures camera"),
            (UserProfile::Designer, "figma sketch photoshop illustrator assets mockups ui ux design"),
            (UserProfile::Writer, "markdown notes obsidian documents writing research draft chapter"),
            (UserProfile::Student, "lecture notes assignment homework semester course university pdf"),
            (UserProfile::Business, "excel spreadsheet report presentation quarterly budget invoice"),
        ];

        // Embeddings vergleichen
        let user_embedding = embedding_engine.embed(&description).ok()?;

        let best_match = prototypes.iter()
            .map(|(profile, proto_text)| {
                let proto_embedding = embedding_engine.embed(proto_text).ok()?;
                let similarity = cosine_similarity(&user_embedding, &proto_embedding);
                Some((profile.clone(), similarity))
            })
            .flatten()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;

        Some(best_match.0)
    }
}

/// Ergebnis der Profil-Erkennung
#[derive(Debug, Clone)]
pub struct ProfileSuggestion {
    /// Wahrscheinlichstes Profil
    pub primary: UserProfile,
    /// Zweitwahrscheinlichstes Profil
    pub secondary: Option<UserProfile>,
    /// Konfidenz (0.0 - 1.0)
    /// - > 0.7: Klar, zeige direkt
    /// - 0.4 - 0.7: Zeige Top-2 zur Auswahl
    /// - < 0.4: Frage den Nutzer (Custom-Flow)
    pub confidence: f32,
    /// Erkannte Apps
    pub detected_apps: Vec<DetectedApp>,
    /// Alle Scores für Debugging
    pub scores: Vec<(UserProfile, i32)>,
}

/// Quick-Scan Ergebnis für Profil-Erkennung
#[derive(Debug, Clone, Default)]
pub struct QuickScanResult {
    /// Anzahl pro Dateiendung
    pub extension_counts: HashMap<String, usize>,
    /// Erkannte Apps/Kataloge
    pub detected_apps: Vec<DetectedApp>,
    /// Sample von Dateipfaden (für Embedding-Klassifikation)
    pub sample_paths: Vec<PathBuf>,
    /// Gesamtzahl gescannter Dateien
    pub total_files: usize,
    /// Scan-Dauer in ms
    pub scan_duration_ms: u64,
}

/// Quick-Scanner für Profil-Erkennung (~5 Sekunden)
pub struct QuickScanner {
    paths: Vec<PathBuf>,
    max_files: usize,
}

impl QuickScanner {
    pub fn new() -> Self {
        Self {
            paths: vec![],
            max_files: 50_000, // Limit für Geschwindigkeit
        }
    }

    pub fn with_paths(mut self, paths: &[PathBuf]) -> Self {
        self.paths = paths.to_vec();
        self
    }

    /// Schneller Metadaten-Scan (nur Dateiendungen zählen)
    pub fn scan(&self) -> Result<QuickScanResult, CxpError> {
        use std::time::Instant;
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

                // App-Erkennung
                if let Some(app) = Self::detect_app(path) {
                    if !result.detected_apps.iter().any(|a| a.path == app.path) {
                        result.detected_apps.push(app);
                    }
                }

                if entry.file_type().is_file() {
                    // Dateiendung zählen
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        *result.extension_counts.entry(ext_str).or_insert(0) += 1;
                    }

                    // Sample für Embedding
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

        // Schnelles Überspringen von bekannten Junk-Ordnern
        matches!(name.as_ref(),
            "node_modules" | ".git" | "target" | "dist" | "build" |
            ".cache" | "__pycache__" | ".venv" | "venv" | ".idea" |
            ".vs" | "Library" | "Caches" | ".Trash"
        )
    }

    fn detect_app(path: &Path) -> Option<DetectedApp> {
        let name = path.file_name()?.to_string_lossy();

        // Lightroom Katalog
        if name.ends_with(".lrcat") {
            return Some(DetectedApp {
                name: name.to_string(),
                app_type: "Lightroom",
                path: path.to_path_buf(),
                importance: 1.0,
            });
        }

        // Obsidian Vault
        if name == ".obsidian" && path.is_dir() {
            return Some(DetectedApp {
                name: path.parent()?.file_name()?.to_string_lossy().to_string(),
                app_type: "Obsidian",
                path: path.parent()?.to_path_buf(),
                importance: 1.0,
            });
        }

        // Git Repository
        if name == ".git" && path.is_dir() {
            return Some(DetectedApp {
                name: path.parent()?.file_name()?.to_string_lossy().to_string(),
                app_type: "Git Repository",
                path: path.parent()?.to_path_buf(),
                importance: 0.8,
            });
        }

        None
    }
}
```

### Custom-Profil Konfiguration

```rust
// scanner/custom_config.rs

/// Nutzer-definierte Konfiguration (wenn kein Profil passt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    /// Was erstellt der Nutzer?
    pub content_types: ContentTypes,

    /// Erkannte + manuell hinzugefügte Apps
    pub apps: Vec<String>,

    /// Wichtige Ordner
    pub watched_folders: Vec<PathBuf>,

    /// Dateigrößen-Limit in MB
    pub max_file_size_mb: u64,

    /// Bilder mit KI-Suche indexieren?
    pub enable_image_search: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentTypes {
    pub code: bool,
    pub photos: bool,
    pub designs: bool,
    pub documents: bool,
    pub spreadsheets: bool,
}

impl CustomConfig {
    /// Konvertiert CustomConfig zu ScanConfig
    pub fn to_scan_config(&self) -> ScanConfig {
        let mut extensions = Vec::new();

        if self.content_types.code {
            extensions.extend(["rs", "ts", "tsx", "js", "jsx", "py", "go", "java",
                             "cpp", "c", "h", "json", "yaml", "toml", "md"]);
        }
        if self.content_types.photos {
            extensions.extend(["raw", "cr2", "cr3", "nef", "arw", "dng",
                             "jpg", "jpeg", "png", "heic", "heif", "tiff"]);
        }
        if self.content_types.designs {
            extensions.extend(["fig", "sketch", "psd", "ai", "xd", "indd", "svg", "pdf"]);
        }
        if self.content_types.documents {
            extensions.extend(["md", "txt", "docx", "doc", "pdf", "odt", "rtf", "tex"]);
        }
        if self.content_types.spreadsheets {
            extensions.extend(["xlsx", "xls", "csv", "pptx", "ppt"]);
        }

        ScanConfig {
            paths: self.watched_folders.clone(),
            file_extensions: extensions.into_iter().map(String::from).collect(),
            max_file_size: self.max_file_size_mb * 1024 * 1024,
            include_images: self.enable_image_search,
            ..Default::default()
        }
    }
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            content_types: ContentTypes::default(),
            apps: vec![],
            watched_folders: vec![
                dirs::document_dir().unwrap_or_default(),
                dirs::desktop_dir().unwrap_or_default(),
            ],
            max_file_size_mb: 50,
            enable_image_search: false,
        }
    }
}
```

---

## Architektur

```
cxp-core/src/
├── scanner/
│   ├── mod.rs              # Public API
│   ├── config.rs           # ScanConfig, Presets
│   ├── ignore.rs           # Ignore-Patterns (globset)
│   ├── detector.rs         # Projekt/Typ-Erkennung
│   ├── relevance.rs        # Aktivitäts-Scoring
│   └── tier.rs             # HOT/WARM/COLD Kategorisierung
```

---

## Phase 1: Ignore-System

### 1.1 Immer ignorieren (hardcoded)

```rust
// scanner/ignore.rs

/// Diese Patterns werden IMMER ignoriert - kein Override möglich
pub const ALWAYS_IGNORE: &[&str] = &[
    // === DEPENDENCIES (nicht dein Code) ===
    "node_modules/**",
    "**/node_modules/**",
    "vendor/**",
    ".venv/**",
    "venv/**",
    "__pycache__/**",
    "**/__pycache__/**",
    ".tox/**",
    "*.egg-info/**",

    // === BUILD OUTPUT (regenerierbar) ===
    "target/debug/**",
    "target/release/**",
    "dist/**",
    "build/**",
    ".next/**",
    ".nuxt/**",
    ".output/**",
    "out/**",

    // === GIT INTERNALS (binär, riesig) ===
    ".git/objects/**",
    ".git/lfs/**",

    // === CACHES (temporär) ===
    ".cache/**",
    "**/.cache/**",
    ".pytest_cache/**",
    ".mypy_cache/**",
    ".ruff_cache/**",
    "*.pyc",
    "*.pyo",
    ".sass-cache/**",
    ".parcel-cache/**",

    // === IDE (persönlich) ===
    ".idea/**",
    ".vs/**",
    "*.swp",
    "*.swo",
    "*~",

    // === BINARIES (nicht durchsuchbar) ===
    "*.exe",
    "*.dll",
    "*.so",
    "*.dylib",
    "*.wasm",
    "*.o",
    "*.obj",
    "*.a",
    "*.lib",

    // === ARCHIVE (bereits komprimiert) ===
    "*.zip",
    "*.tar",
    "*.gz",
    "*.bz2",
    "*.xz",
    "*.rar",
    "*.7z",
    "*.dmg",
    "*.iso",

    // === LOGS (temporär, meist riesig) ===
    "*.log",
    "logs/**",
    "*.log.*",

    // === LOCK FILES (regenerierbar) ===
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "poetry.lock",
    "Pipfile.lock",

    // === GENERATED (regenerierbar) ===
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
```

### 1.2 Default ignorieren (überschreibbar)

```rust
/// Diese Patterns werden standardmäßig ignoriert, können aber aktiviert werden
pub const DEFAULT_IGNORE: &[&str] = &[
    // Oft nicht relevant, aber manchmal gewollt
    ".vscode/**",      // VS Code settings
    ".github/**",      // GitHub workflows
    "docs/**",         // Manchmal gewollt
    "test/**",         // Test-Dateien
    "tests/**",
    "spec/**",
    "__tests__/**",
    "*.test.*",
    "*.spec.*",
];
```

### 1.3 Konfigurierbare Patterns

```rust
pub struct IgnoreConfig {
    /// Zusätzliche Ignore-Patterns vom Nutzer
    pub custom_ignore: Vec<String>,

    /// Patterns die trotz Default-Ignore eingeschlossen werden
    pub force_include: Vec<String>,

    /// Max Dateigröße (default: 10 MB)
    pub max_file_size: u64,

    /// Versteckte Dateien einschließen (default: false)
    pub include_hidden: bool,
}
```

---

## Phase 2: Projekt-Erkennung

### 2.1 Projekt-Root Erkennung

```rust
// scanner/detector.rs

pub struct ProjectDetector;

impl ProjectDetector {
    /// Erkennt Projekt-Root anhand von Marker-Dateien
    pub fn detect_project_root(path: &Path) -> Option<ProjectInfo> {
        // Priorität der Marker
        let markers = [
            // Rust
            ("Cargo.toml", ProjectType::Rust),
            // JavaScript/TypeScript
            ("package.json", ProjectType::JavaScript),
            // Python
            ("pyproject.toml", ProjectType::Python),
            ("setup.py", ProjectType::Python),
            ("requirements.txt", ProjectType::Python),
            // Go
            ("go.mod", ProjectType::Go),
            // Git (generisch)
            (".git", ProjectType::Generic),
        ];

        for (marker, project_type) in markers {
            if path.join(marker).exists() {
                return Some(ProjectInfo {
                    root: path.to_path_buf(),
                    project_type,
                    name: path.file_name()?.to_string_lossy().to_string(),
                });
            }
        }
        None
    }

    /// Findet alle Projekte in einem Verzeichnis
    pub fn find_all_projects(base: &Path) -> Vec<ProjectInfo> {
        // Rekursiv suchen, aber nicht in Unterordner von Projekten
        // (verhindert nested project detection)
    }
}

pub struct ProjectInfo {
    pub root: PathBuf,
    pub project_type: ProjectType,
    pub name: String,
}

pub enum ProjectType {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    Generic,
}
```

### 2.2 Datei-Typ Kategorisierung

```rust
pub enum FileCategory {
    /// Eigener Source Code
    SourceCode,
    /// Konfigurationsdateien
    Config,
    /// Dokumentation
    Documentation,
    /// Test-Dateien
    Tests,
    /// Bilder/Media
    Images,
    /// Daten (JSON, CSV, etc.)
    Data,
    /// Unbekannt
    Unknown,
}

impl FileCategory {
    pub fn from_path(path: &Path, project_type: &ProjectType) -> Self {
        // Intelligente Kategorisierung basierend auf:
        // - Dateiendung
        // - Pfad (src/, tests/, docs/)
        // - Projekt-Typ
    }
}
```

---

## Phase 3: Aktivitäts-Scoring

### 3.1 Relevanz berechnen

```rust
// scanner/relevance.rs

pub struct RelevanceScorer;

impl RelevanceScorer {
    /// Berechnet Relevanz-Score (0.0 - 1.0)
    pub fn calculate(path: &Path, project: &ProjectInfo) -> f32 {
        let mut score = 0.0;

        // Faktor 1: Letzte Änderung (40%)
        let modified_score = Self::modified_score(path);
        score += modified_score * 0.4;

        // Faktor 2: Wichtigkeit basierend auf Dateiname (30%)
        let importance_score = Self::importance_score(path);
        score += importance_score * 0.3;

        // Faktor 3: Tiefe im Projekt (15%)
        // Dateien näher am Root sind wichtiger
        let depth_score = Self::depth_score(path, &project.root);
        score += depth_score * 0.15;

        // Faktor 4: Dateigröße (15%)
        // Sehr kleine oder sehr große Dateien sind oft weniger relevant
        let size_score = Self::size_score(path);
        score += size_score * 0.15;

        score.clamp(0.0, 1.0)
    }

    fn modified_score(path: &Path) -> f32 {
        let metadata = path.metadata().ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;

        match age.as_secs() {
            0..=86400 => 1.0,           // < 1 Tag
            0..=604800 => 0.9,          // < 1 Woche
            0..=2592000 => 0.7,         // < 30 Tage
            0..=7776000 => 0.5,         // < 90 Tage
            0..=31536000 => 0.3,        // < 1 Jahr
            _ => 0.1,                    // > 1 Jahr
        }
    }

    fn importance_score(path: &Path) -> f32 {
        let name = path.file_name()?.to_string_lossy().to_lowercase();

        // Sehr wichtige Dateien
        if matches!(name.as_str(),
            "readme.md" | "readme" | "changelog.md" | "license" |
            "main.rs" | "lib.rs" | "mod.rs" |
            "index.ts" | "index.js" | "app.tsx" | "app.ts" |
            "main.py" | "__init__.py" | "app.py"
        ) {
            return 1.0;
        }

        // Wichtige Configs
        if matches!(name.as_str(),
            "package.json" | "cargo.toml" | "pyproject.toml" |
            "tsconfig.json" | ".env.example"
        ) {
            return 0.9;
        }

        // Source Code
        if path.extension().map(|e| is_text_file(e.to_str().unwrap_or(""))).unwrap_or(false) {
            return 0.7;
        }

        0.5
    }
}
```

### 3.2 Tier-System

```rust
// scanner/tier.rs

pub enum Tier {
    /// Immer geladen, sofort verfügbar
    Hot,
    /// On-Demand laden
    Warm,
    /// Archiv, selten gebraucht
    Cold,
    /// Wird ignoriert
    Ignore,
}

impl Tier {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.7 => Tier::Hot,
            s if s >= 0.4 => Tier::Warm,
            s if s >= 0.1 => Tier::Cold,
            _ => Tier::Ignore,
        }
    }
}

pub struct TierManager {
    pub hot_files: Vec<PathBuf>,
    pub warm_files: Vec<PathBuf>,
    pub cold_files: Vec<PathBuf>,
}

impl TierManager {
    /// Kategorisiert alle Dateien in Tiers
    pub fn categorize(files: &[ScannedFile]) -> Self {
        let mut hot = Vec::new();
        let mut warm = Vec::new();
        let mut cold = Vec::new();

        for file in files {
            match Tier::from_score(file.relevance) {
                Tier::Hot => hot.push(file.path.clone()),
                Tier::Warm => warm.push(file.path.clone()),
                Tier::Cold => cold.push(file.path.clone()),
                Tier::Ignore => {},
            }
        }

        Self { hot_files: hot, warm_files: warm, cold_files: cold }
    }
}
```

---

## Phase 4: KI-Integration (< $0.10)

### 4.1 Wann KI nutzen?

```rust
pub enum AiDecision {
    /// Automatisch entscheiden (kein KI-Call)
    Auto,
    /// KI fragen (kostet Tokens)
    AskAi,
}

impl AiDecision {
    /// Entscheidet ob KI gefragt werden soll
    pub fn should_ask(context: &ScanContext) -> Self {
        // KI NUR fragen wenn:
        // 1. Unklare Situation (nicht eindeutig ignorieren/include)
        // 2. Große Auswirkung (viele Dateien betroffen)
        // 3. Einmalige Entscheidung (wird gespeichert)

        if context.unclear_projects > 0 && context.total_files > 1000 {
            AiDecision::AskAi
        } else {
            AiDecision::Auto
        }
    }
}
```

### 4.2 KI-Prompt Strategie (Token-optimiert)

```rust
pub struct AiScanner {
    /// Max Tokens für gesamten Scan-Prozess
    max_tokens: usize,  // Default: 5000 (~$0.05)
}

impl AiScanner {
    /// Generiert kompakten Prompt für Projekt-Übersicht
    pub fn generate_overview_prompt(projects: &[ProjectInfo]) -> String {
        // KOMPAKT! Nur Projektnamen + Typ + Größe
        // Keine Dateilisten, keine Details
        format!(
            "Found {} projects:\n{}\n\nWhich are relevant for AI context? Reply with project names only.",
            projects.len(),
            projects.iter()
                .map(|p| format!("- {} ({:?}, {} files)", p.name, p.project_type, p.file_count))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Parst KI-Antwort
    pub fn parse_response(response: &str, projects: &[ProjectInfo]) -> Vec<String> {
        // Extrahiert Projektnamen aus Antwort
    }
}
```

### 4.3 Kosten-Kontrolle

```rust
pub struct CostTracker {
    /// Bereits verbrauchte Tokens
    used_tokens: usize,
    /// Budget in Tokens
    budget_tokens: usize,  // ~50000 = $0.10
}

impl CostTracker {
    pub fn can_afford(&self, estimated_tokens: usize) -> bool {
        self.used_tokens + estimated_tokens <= self.budget_tokens
    }

    pub fn track(&mut self, tokens: usize) {
        self.used_tokens += tokens;
        tracing::info!(
            "AI tokens used: {}/{} (~${:.4})",
            self.used_tokens,
            self.budget_tokens,
            self.used_tokens as f64 * 0.000002  // GPT-4 pricing approx
        );
    }
}
```

---

## Phase 5: Public API

### 5.1 SmartScanner Struct

```rust
// scanner/mod.rs

pub struct SmartScanner {
    config: ScanConfig,
    ignore_matcher: GlobSet,
    cost_tracker: CostTracker,
}

pub struct ScanConfig {
    /// Zu scannende Pfade
    pub paths: Vec<PathBuf>,

    /// Ignore-Konfiguration
    pub ignore: IgnoreConfig,

    /// KI für Entscheidungen nutzen (default: false)
    pub use_ai: bool,

    /// Max Budget für KI in USD (default: 0.10)
    pub ai_budget_usd: f64,

    /// Automatisch nach Tiers kategorisieren
    pub auto_tier: bool,

    /// Bilder einschließen
    pub include_images: bool,
}

pub struct ScanResult {
    /// Alle gescannten Dateien
    pub files: Vec<ScannedFile>,

    /// Erkannte Projekte
    pub projects: Vec<ProjectInfo>,

    /// Tier-Kategorisierung
    pub tiers: TierManager,

    /// Statistiken
    pub stats: ScanStats,

    /// KI-Kosten (falls genutzt)
    pub ai_cost_usd: f64,
}

pub struct ScannedFile {
    pub path: PathBuf,
    pub size: u64,
    pub category: FileCategory,
    pub tier: Tier,
    pub relevance: f32,
    pub modified: SystemTime,
}

pub struct ScanStats {
    pub total_scanned: usize,
    pub total_included: usize,
    pub total_ignored: usize,
    pub total_size_bytes: u64,
    pub by_tier: HashMap<Tier, usize>,
    pub by_category: HashMap<FileCategory, usize>,
    pub by_project: HashMap<String, usize>,
}
```

### 5.2 Builder Pattern

```rust
impl SmartScanner {
    pub fn new() -> Self { ... }

    /// Pfade hinzufügen
    pub fn with_paths<P: AsRef<Path>>(mut self, paths: &[P]) -> Self { ... }

    /// Eigene Ignore-Patterns
    pub fn with_ignore(mut self, patterns: &[&str]) -> Self { ... }

    /// Force-Include Patterns
    pub fn with_include(mut self, patterns: &[&str]) -> Self { ... }

    /// KI aktivieren
    pub fn with_ai(mut self, budget_usd: f64) -> Self { ... }

    /// Bilder einschließen
    pub fn with_images(mut self) -> Self { ... }

    /// Scan ausführen
    pub fn scan(self) -> Result<ScanResult> { ... }
}
```

### 5.3 Integration mit CxpBuilder

```rust
impl CxpBuilder {
    /// Nutzt SmartScanner statt einfachem walkdir
    pub fn from_scan(scan_result: ScanResult) -> Self {
        let mut builder = Self::new_empty();

        // Nur HOT + WARM Dateien einschließen (COLD optional)
        for file in scan_result.files {
            if matches!(file.tier, Tier::Hot | Tier::Warm) {
                builder.add_file(&file.path);
            }
        }

        builder
    }

    /// Erstellt separate CXPs pro Tier
    pub fn from_scan_tiered(scan_result: ScanResult) -> TieredCxpBuilder {
        TieredCxpBuilder {
            hot: Self::from_files(&scan_result.tiers.hot_files),
            warm: Self::from_files(&scan_result.tiers.warm_files),
            cold: Self::from_files(&scan_result.tiers.cold_files),
        }
    }
}

pub struct TieredCxpBuilder {
    pub hot: CxpBuilder,
    pub warm: CxpBuilder,
    pub cold: CxpBuilder,
}

impl TieredCxpBuilder {
    pub fn build_all<P: AsRef<Path>>(self, base_path: P) -> Result<()> {
        let base = base_path.as_ref();
        self.hot.build(base.join("hot.cxp"))?;
        self.warm.build(base.join("warm.cxp"))?;
        self.cold.build(base.join("cold.cxp"))?;
        Ok(())
    }
}
```

---

## Phase 6: CLI Erweiterungen

```bash
# Smart Scan mit Vorschau
cxp smart-scan /path/to/folder
# Zeigt: Gefundene Projekte, Dateien pro Tier, geschätzte Größe

# Mit KI-Unterstützung
cxp smart-scan /path/to/folder --ai --budget 0.05

# Nur bestimmte Kategorien
cxp smart-scan /path/to/folder --only code,docs

# Tiered Build
cxp smart-build /path/to/folder --output ./data/
# Erstellt: hot.cxp, warm.cxp, cold.cxp

# Einzelne Tier
cxp smart-build /path/to/folder --tier hot --output active.cxp
```

---

## Implementierungs-Reihenfolge

### Schritt 1: Basis (Keine KI)
- [ ] `scanner/mod.rs` - Modul-Struktur
- [ ] `scanner/ignore.rs` - ALWAYS_IGNORE + DEFAULT_IGNORE
- [ ] `scanner/config.rs` - ScanConfig
- [ ] Cargo.toml: `globset = "0.4"`
- [ ] Tests für Ignore-Patterns

### Schritt 2: Erkennung
- [ ] `scanner/detector.rs` - Projekt-Erkennung
- [ ] `scanner/relevance.rs` - Aktivitäts-Scoring
- [ ] `scanner/tier.rs` - HOT/WARM/COLD

### Schritt 3: Integration
- [ ] `SmartScanner` Public API
- [ ] Integration mit `CxpBuilder`
- [ ] CLI Commands

### Schritt 4: KI (Optional)
- [ ] `AiScanner` für Entscheidungen
- [ ] `CostTracker` für Budget
- [ ] Prompt-Optimierung

---

## Cargo.toml Änderungen

```toml
[features]
default = []
scanner = ["globset"]
scanner-ai = ["scanner"]  # KI-Features

[dependencies]
globset = { version = "0.4", optional = true }
```

---

## Erwartete Ergebnisse

| Szenario | Ohne SmartScanner | Mit SmartScanner |
|----------|-------------------|------------------|
| 130 GB PC | 238.000 Dateien | ~15.000 Dateien |
| CXP Größe | 800 MB | ~100 MB |
| Relevanz | 5% (viel Rauschen) | 95% (fokussiert) |
| Build-Zeit | 3+ Stunden | ~10 Minuten |
| KI-Kosten | - | < $0.10 |
