//! Zstandard compression for chunks
//!
//! Provides efficient compression with good speed/ratio trade-off.

use crate::{CxpError, Result};
use zstd::stream::{encode_all, decode_all};
use std::io::Cursor;

/// Default compression level (3 is a good balance)
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

/// Compress data using Zstandard
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    compress_with_level(data, DEFAULT_COMPRESSION_LEVEL)
}

/// Compress data with a specific compression level (1-22)
pub fn compress_with_level(data: &[u8], level: i32) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    encode_all(cursor, level).map_err(|e| CxpError::Compression(e.to_string()))
}

/// Decompress Zstandard compressed data
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    decode_all(cursor).map_err(|e| CxpError::Compression(e.to_string()))
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
}

impl CompressionStats {
    /// Calculate compression ratio (compressed / original)
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }

    /// Calculate space savings percentage
    pub fn savings_percent(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        (1.0 - self.ratio()) * 100.0
    }
}

/// Compress data and return stats along with compressed data
pub fn compress_with_stats(data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
    let compressed = compress(data)?;
    let stats = CompressionStats {
        original_size: data.len(),
        compressed_size: compressed.len(),
    };
    Ok((compressed, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = b"Hello, World! This is test data for compression.";
        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(original.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compression_ratio() {
        // Highly compressible data (repeated pattern)
        let data: Vec<u8> = (0..1000).map(|i| (i % 10) as u8).collect();
        let (compressed, stats) = compress_with_stats(&data).unwrap();

        assert!(compressed.len() < data.len());
        assert!(stats.savings_percent() > 0.0);
    }

    #[test]
    fn test_empty_data() {
        let original = b"";
        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(original.as_slice(), decompressed.as_slice());
    }
}
