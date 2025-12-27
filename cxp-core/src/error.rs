//! Error types for CXP operations

use thiserror::Error;

/// CXP Error types
#[derive(Error, Debug)]
pub enum CxpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid CXP file: {0}")]
    InvalidFormat(String),

    #[error("Chunk error: {0}")]
    Chunk(String),

    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Index error: {0}")]
    Index(String),

    #[error("Search error: {0}")]
    Search(String),
}

/// Result type for CXP operations
pub type Result<T> = std::result::Result<T, CxpError>;

impl From<rmp_serde::encode::Error> for CxpError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        CxpError::Serialization(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for CxpError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        CxpError::Serialization(e.to_string())
    }
}

impl From<serde_json::Error> for CxpError {
    fn from(e: serde_json::Error) -> Self {
        CxpError::Serialization(e.to_string())
    }
}

#[cfg(any(feature = "embeddings", feature = "multimodal"))]
impl From<ort::Error> for CxpError {
    fn from(e: ort::Error) -> Self {
        CxpError::Embedding(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CxpError::FileNotFound("test.txt".to_string());
        assert_eq!(err.to_string(), "File not found: test.txt");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let cxp_err: CxpError = io_err.into();
        assert!(matches!(cxp_err, CxpError::Io(_)));
    }

    #[test]
    fn test_error_compression() {
        let err = CxpError::Compression("compression failed".to_string());
        assert!(err.to_string().contains("compression failed"));
    }

    #[test]
    fn test_error_serialization() {
        let err = CxpError::Serialization("invalid format".to_string());
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            CxpError::Serialization("test".into()),
            CxpError::InvalidFormat("test".into()),
            CxpError::Chunk("test".into()),
            CxpError::Manifest("test".into()),
            CxpError::FileNotFound("test".into()),
            CxpError::UnsupportedFileType("test".into()),
            CxpError::Compression("test".into()),
            CxpError::Embedding("test".into()),
            CxpError::Index("test".into()),
            CxpError::Search("test".into()),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
