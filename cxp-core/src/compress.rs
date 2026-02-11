//! Zstandard compression for chunks
//!
//! Provides efficient compression with good speed/ratio trade-off.
//! Supports dictionary-based compression for improved efficiency on small chunks.

use crate::{CxpError, Result};
use zstd::stream::{encode_all, decode_all};
use zstd::dict::{EncoderDictionary, DecoderDictionary};
use std::io::Cursor;

/// Default compression level (3 is a good balance)
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

/// Ultra compression level for metadata (Zstd level 19-22)
pub const ULTRA_COMPRESSION_LEVEL: i32 = 19;

/// Compress data using Zstandard with Ultra settings
pub fn compress_ultra(data: &[u8]) -> Result<Vec<u8>> {
    compress_with_level(data, ULTRA_COMPRESSION_LEVEL)
}
/// Compress data using Zstandard with default settings
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    compress_with_level(data, DEFAULT_COMPRESSION_LEVEL)
}

/// Compress data with a specific compression level (1-22)
pub fn compress_with_level(data: &[u8], level: i32) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    encode_all(cursor, level).map_err(|e| CxpError::Compression(e.to_string()))
}

/// Compress data using a Zstandard dictionary
pub fn compress_with_dict(data: &[u8], dict: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let dict = EncoderDictionary::copy(dict, level);
    let mut encoder = zstd::stream::Encoder::with_prepared_dictionary(&mut result, &dict)
        .map_err(|e| CxpError::Compression(e.to_string()))?;
    
    std::io::copy(&mut Cursor::new(data), &mut encoder)
        .map_err(|e| CxpError::Compression(e.to_string()))?;
    
    encoder.finish().map_err(|e| CxpError::Compression(e.to_string()))?;
    Ok(result)
}

/// Decompress Zstandard compressed data
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    decode_all(cursor).map_err(|e| CxpError::Compression(e.to_string()))
}

/// Decompress data using a Zstandard dictionary
pub fn decompress_with_dict(data: &[u8], dict: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let dict = DecoderDictionary::copy(dict);
    let mut decoder = zstd::stream::Decoder::with_prepared_dictionary(Cursor::new(data), &dict)
        .map_err(|e| CxpError::Compression(e.to_string()))?;
    
    std::io::copy(&mut decoder, &mut result)
        .map_err(|e| CxpError::Compression(e.to_string()))?;
    
    Ok(result)
}

/// Train a Zstandard dictionary from a collection of samples
pub fn train_dictionary(samples: &[Vec<u8>], capacity: usize) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut sizes = Vec::new();
    for sample in samples {
        data.extend_from_slice(sample);
        sizes.push(sample.len());
    }
    
    zstd::dict::from_continuous(&data, &sizes, capacity)
        .map_err(|e| CxpError::Compression(format!("Failed to train dictionary: {}", e)))
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
