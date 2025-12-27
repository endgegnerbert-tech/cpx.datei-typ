//! CXP - Universal AI Context Format
//!
//! A KI-optimized data format that is 85% smaller than JSON,
//! with built-in semantic search and multi-platform support.

pub mod chunker;
pub mod dedup;
pub mod compress;
pub mod format;
pub mod manifest;
pub mod error;
pub mod extensions;

#[cfg(feature = "contextai")]
pub mod contextai;

#[cfg(any(feature = "embeddings", feature = "embeddings-wasm", feature = "multimodal"))]
pub mod embeddings;

#[cfg(feature = "embeddings-wasm")]
pub mod embeddings_tract;

#[cfg(feature = "multimodal")]
pub mod multimodal;

#[cfg(feature = "search")]
pub mod index;

#[cfg(all(feature = "search", feature = "multimodal"))]
pub mod unified_index;

#[cfg(any(feature = "embeddings", feature = "embeddings-wasm", feature = "multimodal"))]
pub mod semantic;

pub use error::{CxpError, Result};
pub use manifest::Manifest;
pub use format::{CxpFile, CxpBuilder, CxpReader};
pub use extensions::{Extension, ExtensionManager, ExtensionManifest};

#[cfg(feature = "contextai")]
pub use contextai::ContextAIExtension;

// Export common embedding types from either feature
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm", feature = "multimodal"))]
pub use embeddings::{EmbeddingModel, BinaryEmbedding, Int8Embedding, QuantizedEmbeddings};

// Export native engine (ort-based)
#[cfg(feature = "embeddings")]
pub use embeddings::EmbeddingEngine;

// Export WASM engine (tract-based)
#[cfg(feature = "embeddings-wasm")]
pub use embeddings_tract::TractEmbeddingEngine;

// Export multimodal engine
#[cfg(feature = "multimodal")]
pub use multimodal::{MultimodalEngine, SIGLIP2_DIMENSIONS, cosine_similarity, cosine_distance};

// Export search types
#[cfg(feature = "search")]
pub use index::{HnswIndex, HnswConfig, DistanceMetric, SearchResult};

// Export unified index types
#[cfg(all(feature = "search", feature = "multimodal"))]
pub use unified_index::{UnifiedIndex, EntryType, SearchResultWithType};

// Export semantic storage types
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm", feature = "multimodal"))]
pub use semantic::{
    EmbeddingStore,
    serialize_binary_embeddings,
    deserialize_binary_embeddings,
    serialize_int8_embeddings,
    deserialize_int8_embeddings,
};

/// CXP Format Version
pub const VERSION: &str = "1.0.0";

/// Default chunk size boundaries (in bytes)
pub const MIN_CHUNK_SIZE: u32 = 2 * 1024;      // 2 KB
pub const AVG_CHUNK_SIZE: u32 = 4 * 1024;      // 4 KB
pub const MAX_CHUNK_SIZE: u32 = 8 * 1024;      // 8 KB

/// Supported file extensions for text content
pub const TEXT_EXTENSIONS: &[&str] = &[
    // Code
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "c", "cpp", "h", "hpp",
    "cs", "rb", "php", "swift", "kt", "scala", "r", "sql", "sh", "bash", "zsh",
    "ps1", "bat", "cmd",
    // Config
    "json", "yaml", "yml", "toml", "xml", "ini", "env", "conf", "config",
    // Docs
    "md", "mdx", "txt", "rst", "adoc", "tex",
    // Web
    "html", "htm", "css", "scss", "sass", "less", "vue", "svelte",
    // Data
    "csv", "tsv",
];

/// Supported image extensions for multimodal processing
pub const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif",
];

/// Check if a file extension is a supported text format
pub fn is_text_file(ext: &str) -> bool {
    TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

/// Check if a file extension is a supported image format
pub fn is_image_file(ext: &str) -> bool {
    IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_text_file_rust() {
        assert!(is_text_file("rs"));
        assert!(is_text_file("RS"));
        assert!(is_text_file("Rs"));
    }

    #[test]
    fn test_is_text_file_typescript() {
        assert!(is_text_file("ts"));
        assert!(is_text_file("tsx"));
        assert!(is_text_file("js"));
        assert!(is_text_file("jsx"));
    }

    #[test]
    fn test_is_text_file_config() {
        assert!(is_text_file("json"));
        assert!(is_text_file("yaml"));
        assert!(is_text_file("yml"));
        assert!(is_text_file("toml"));
    }

    #[test]
    fn test_is_text_file_markdown() {
        assert!(is_text_file("md"));
        assert!(is_text_file("mdx"));
    }

    #[test]
    fn test_is_text_file_unsupported() {
        assert!(!is_text_file("exe"));
        assert!(!is_text_file("png"));
        assert!(!is_text_file("jpg"));
        assert!(!is_text_file("pdf"));
        assert!(!is_text_file("zip"));
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file("png"));
        assert!(is_image_file("jpg"));
        assert!(is_image_file("jpeg"));
        assert!(is_image_file("gif"));
        assert!(is_image_file("webp"));
        assert!(is_image_file("PNG"));
        assert!(is_image_file("JPG"));
    }

    #[test]
    fn test_is_image_file_unsupported() {
        assert!(!is_image_file("txt"));
        assert!(!is_image_file("rs"));
        assert!(!is_image_file("pdf"));
        assert!(!is_image_file("exe"));
    }

    #[test]
    fn test_version_format() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_chunk_size_constants() {
        assert!(MIN_CHUNK_SIZE < AVG_CHUNK_SIZE);
        assert!(AVG_CHUNK_SIZE < MAX_CHUNK_SIZE);
        assert_eq!(MIN_CHUNK_SIZE, 2048);
        assert_eq!(AVG_CHUNK_SIZE, 4096);
        assert_eq!(MAX_CHUNK_SIZE, 8192);
    }
}
