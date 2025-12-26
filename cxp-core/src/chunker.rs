//! Content-Defined Chunking using FastCDC
//!
//! Splits files into variable-sized chunks based on content boundaries,
//! which enables efficient deduplication.

use crate::{MIN_CHUNK_SIZE, AVG_CHUNK_SIZE, MAX_CHUNK_SIZE};
use fastcdc::v2020::FastCDC;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// A content-defined chunk with its hash and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// SHA-256 hash of the chunk content (hex encoded)
    pub hash: String,
    /// Chunk data (before compression)
    #[serde(skip)]
    pub data: Vec<u8>,
    /// Original offset in the source file
    pub offset: usize,
    /// Length of the chunk
    pub length: usize,
}

impl Chunk {
    /// Create a new chunk from data
    pub fn new(data: Vec<u8>, offset: usize) -> Self {
        let hash = compute_hash(&data);
        let length = data.len();
        Self { hash, data, offset, length }
    }

    /// Get the chunk ID (first 16 chars of hash)
    pub fn id(&self) -> &str {
        &self.hash[..16]
    }
}

/// Compute SHA-256 hash of data and return as hex string
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Chunk a file's content using FastCDC
pub fn chunk_content(content: &[u8]) -> Vec<Chunk> {
    if content.is_empty() {
        return Vec::new();
    }

    // Use FastCDC for content-defined chunking
    let chunker = FastCDC::new(content, MIN_CHUNK_SIZE, AVG_CHUNK_SIZE, MAX_CHUNK_SIZE);

    chunker
        .map(|chunk| {
            let data = content[chunk.offset..chunk.offset + chunk.length].to_vec();
            Chunk::new(data, chunk.offset)
        })
        .collect()
}

/// Chunk reference - points to a chunk by hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRef {
    /// Hash of the referenced chunk
    pub hash: String,
    /// Offset within the original file
    pub offset: usize,
    /// Length of this chunk
    pub length: usize,
}

impl From<&Chunk> for ChunkRef {
    fn from(chunk: &Chunk) -> Self {
        Self {
            hash: chunk.hash.clone(),
            offset: chunk.offset,
            length: chunk.length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_content() {
        let content = b"Hello, World! This is a test content that should be chunked.";
        let chunks = chunk_content(content);

        // Small content should produce 1 chunk
        assert!(!chunks.is_empty());

        // Verify hash is computed
        assert_eq!(chunks[0].hash.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_compute_hash() {
        let data = b"test";
        let hash = compute_hash(data);
        assert_eq!(hash.len(), 64);

        // Same data should produce same hash
        let hash2 = compute_hash(data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_empty_content() {
        let chunks = chunk_content(b"");
        assert!(chunks.is_empty());
    }
}
