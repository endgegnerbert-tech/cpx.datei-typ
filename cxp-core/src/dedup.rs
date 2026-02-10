//! Deduplication using content-addressed storage
//!
//! Stores each unique chunk only once, identified by its SHA-256 hash.

use crate::chunker::{Chunk, ChunkRef};
use std::collections::HashMap;

/// Chunk store with deduplication
#[derive(Debug, Default, Clone)]
pub struct ChunkStore {
    /// Chunks in order of discovery
    chunks: Vec<Chunk>,
    /// Maps hash to its index in the chunks vector
    hash_to_index: HashMap<String, usize>,
    /// Statistics
    stats: DeduplicationStats,
}

/// Deduplication statistics
#[derive(Debug, Default, Clone)]
pub struct DeduplicationStats {
    /// Total chunks seen
    pub total_chunks: usize,
    /// Unique chunks stored
    pub unique_chunks: usize,
    /// Total bytes before deduplication
    pub total_bytes: usize,
    /// Bytes after deduplication
    pub deduplicated_bytes: usize,
    /// Number of duplicate chunks found
    pub duplicates_found: usize,
}

impl DeduplicationStats {
    /// Calculate space savings percentage
    pub fn savings_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        let saved = self.total_bytes - self.deduplicated_bytes;
        (saved as f64 / self.total_bytes as f64) * 100.0
    }
}

impl ChunkStore {
    /// Create a new empty chunk store
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chunk to the store, returns (index, is_new)
    pub fn add(&mut self, chunk: Chunk) -> (usize, bool) {
        self.stats.total_chunks += 1;
        self.stats.total_bytes += chunk.length;

        if let Some(&index) = self.hash_to_index.get(&chunk.hash) {
            self.stats.duplicates_found += 1;
            (index, false)
        } else {
            let index = self.chunks.len();
            self.stats.unique_chunks += 1;
            self.stats.deduplicated_bytes += chunk.length;
            self.hash_to_index.insert(chunk.hash.clone(), index);
            self.chunks.push(chunk);
            (index, true)
        }
    }

    /// Add multiple chunks and return their indices
    pub fn add_many_indices(&mut self, chunks: Vec<Chunk>) -> Vec<u32> {
        chunks
            .into_iter()
            .map(|chunk| {
                let (index, _) = self.add(chunk);
                index as u32
            })
            .collect()
    }

    /// Add multiple chunks (legacy support)
    pub fn add_many(&mut self, chunks: Vec<Chunk>) -> Vec<ChunkRef> {
        chunks
            .into_iter()
            .map(|chunk| {
                let chunk_ref = ChunkRef::from(&chunk);
                self.add(chunk);
                chunk_ref
            })
            .collect()
    }

    /// Get a chunk by hash
    pub fn get(&self, hash: &str) -> Option<&Chunk> {
        self.hash_to_index.get(hash).map(|&idx| &self.chunks[idx])
    }

    /// Check if a chunk exists
    pub fn contains(&self, hash: &str) -> bool {
        self.hash_to_index.contains_key(hash)
    }

    /// Get all chunks in order
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }

    /// Iterate over all chunks with their hashes
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Chunk)> {
        self.hash_to_index.iter().map(move |(hash, &idx)| (hash, &self.chunks[idx]))
    }

    /// Get the number of unique chunks
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Get deduplication statistics
    pub fn stats(&self) -> &DeduplicationStats {
        &self.stats
    }

    /// Take all chunks out of the store
    pub fn into_chunks(self) -> Vec<Chunk> {
        self.chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::chunk_content;

    #[test]
    fn test_deduplication() {
        let mut store = ChunkStore::new();

        // Add same content twice
        let content = b"This is test content for deduplication";
        let chunks1 = chunk_content(content);
        let chunks2 = chunk_content(content);

        for chunk in chunks1 {
            let (_, is_new) = store.add(chunk);
            assert!(is_new); // First time: new
        }

        for chunk in chunks2 {
            let (_, is_new) = store.add(chunk);
            assert!(!is_new); // Second time: duplicate
        }

        let stats = store.stats();
        assert!(stats.duplicates_found > 0);
    }

    #[test]
    fn test_stats() {
        let mut store = ChunkStore::new();

        // Add some chunks
        let chunk1 = Chunk::new(vec![1, 2, 3, 4, 5], 0);
        let chunk2 = Chunk::new(vec![6, 7, 8, 9, 10], 5);

        store.add(chunk1);
        store.add(chunk2);

        let stats = store.stats();
        assert_eq!(stats.unique_chunks, 2);
        assert_eq!(stats.total_bytes, 10);
    }
}
