//! Smart Scanner Module for CXP
//!
//! Provides intelligent file scanning with:
//! - User profile detection (Developer, Photographer, Designer, etc.)
//! - Automatic ignore patterns
//! - Relevance scoring
//! - Tier categorization (HOT/WARM/COLD)

mod profile;
mod profile_detector;
mod custom_config;
mod ignore;
mod relevance;
mod tier;
mod config;

pub use profile::{UserProfile, SpecialDetector, DetectedApp};
pub use profile_detector::{ProfileDetector, ProfileSuggestion, QuickScanner, QuickScanResult};
pub use custom_config::{CustomConfig, ContentTypes};
pub use ignore::{IgnoreConfig, ALWAYS_IGNORE, DEFAULT_IGNORE};
pub use relevance::{RelevanceScorer, FileMetadata};
pub use tier::{Tier, TierManager};
pub use config::ScanConfig;
