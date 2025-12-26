//! HNSW Vector Search Index
//!
//! High-performance vector search using USearch's HNSW implementation.
//! Supports both binary embeddings (Hamming distance) and float32 embeddings (cosine similarity).
//!
//! Features:
//! - Fast approximate nearest neighbor search
//! - Persistent index with save/load
//! - Support for binary and float32 embeddings
//! - Integration with CXP embedding types

#[cfg(feature = "search")]
use crate::{CxpError, Result};

#[cfg(all(feature = "search", feature = "embeddings"))]
use crate::{BinaryEmbedding, Int8Embedding};

#[cfg(feature = "search")]
use std::path::Path;

#[cfg(feature = "search")]
use usearch::{Index, MetricKind, ScalarKind};

/// Distance metric for vector similarity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Hamming distance for binary embeddings
    Hamming,
    /// Cosine similarity for float embeddings
    Cosine,
    /// Euclidean (L2) distance
    L2,
    /// Inner product
    IP,
}

impl DistanceMetric {
    /// Convert to USearch MetricKind
    fn to_usearch_metric(&self) -> MetricKind {
        match self {
            DistanceMetric::Hamming => MetricKind::Hamming,
            DistanceMetric::Cosine => MetricKind::Cos,
            DistanceMetric::L2 => MetricKind::L2sq,
            DistanceMetric::IP => MetricKind::IP,
        }
    }
}

/// Configuration for HNSW index
#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// Number of dimensions
    pub dimensions: usize,
    /// Distance metric
    pub metric: DistanceMetric,
    /// Number of connections per layer (M parameter)
    /// Higher values = better recall, more memory
    pub connectivity: usize,
    /// Size of the dynamic candidate list (ef_construction)
    /// Higher values = better index quality, slower construction
    pub expansion_add: usize,
    /// Size of the search queue (ef_search)
    /// Higher values = better recall, slower search
    pub expansion_search: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            dimensions: 384,
            metric: DistanceMetric::Cosine,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
        }
    }
}

impl HnswConfig {
    /// Create config for binary embeddings with Hamming distance
    pub fn binary(dimensions: usize) -> Self {
        Self {
            dimensions: (dimensions + 7) / 8, // Convert bits to bytes
            metric: DistanceMetric::Hamming,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
        }
    }

    /// Create config for float32 embeddings with cosine similarity
    pub fn float32_cosine(dimensions: usize) -> Self {
        Self {
            dimensions,
            metric: DistanceMetric::Cosine,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
        }
    }

    /// Create config for float32 embeddings with L2 distance
    pub fn float32_l2(dimensions: usize) -> Self {
        Self {
            dimensions,
            metric: DistanceMetric::L2,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
        }
    }
}

/// HNSW vector search index
#[cfg(feature = "search")]
pub struct HnswIndex {
    /// USearch index
    index: Index,
    /// Configuration
    config: HnswConfig,
    /// Scalar type (B1 for binary, F32 for float)
    scalar_kind: ScalarKind,
}

#[cfg(feature = "search")]
impl HnswIndex {
    /// Create a new HNSW index with the given configuration
    pub fn new(config: HnswConfig) -> Result<Self> {
        let scalar_kind = match config.metric {
            DistanceMetric::Hamming => ScalarKind::B1,
            _ => ScalarKind::F32,
        };

        let options = usearch::IndexOptions {
            dimensions: config.dimensions,
            metric: config.metric.to_usearch_metric(),
            quantization: scalar_kind,
            connectivity: config.connectivity,
            expansion_add: config.expansion_add,
            expansion_search: config.expansion_search,
        };

        let index = Index::new(&options)
            .map_err(|e| CxpError::Search(format!("Failed to create index: {}", e)))?;

        Ok(Self {
            index,
            config,
            scalar_kind,
        })
    }

    /// Add a float32 vector to the index
    pub fn add_f32(&mut self, id: u64, vector: &[f32]) -> Result<()> {
        if vector.len() != self.config.dimensions {
            return Err(CxpError::Search(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.config.dimensions,
                vector.len()
            )));
        }

        self.index
            .add(id, vector)
            .map_err(|e| CxpError::Search(format!("Failed to add vector: {}", e)))?;

        Ok(())
    }

    /// Add a binary vector (as bytes) to the index
    pub fn add_binary(&mut self, id: u64, bits: &[u8]) -> Result<()> {
        if bits.len() != self.config.dimensions {
            return Err(CxpError::Search(format!(
                "Binary vector size mismatch: expected {} bytes, got {}",
                self.config.dimensions,
                bits.len()
            )));
        }

        self.index
            .add(id, bits)
            .map_err(|e| CxpError::Search(format!("Failed to add binary vector: {}", e)))?;

        Ok(())
    }

    /// Add a BinaryEmbedding to the index
    #[cfg(feature = "embeddings")]
    pub fn add_binary_embedding(&mut self, id: u64, embedding: &BinaryEmbedding) -> Result<()> {
        self.add_binary(id, &embedding.bits)
    }

    /// Add multiple float32 vectors in batch
    pub fn add_batch_f32(&mut self, ids: &[u64], vectors: &[Vec<f32>]) -> Result<()> {
        if ids.len() != vectors.len() {
            return Err(CxpError::Search(
                "IDs and vectors length mismatch".to_string(),
            ));
        }

        for (id, vector) in ids.iter().zip(vectors.iter()) {
            self.add_f32(*id, vector)?;
        }

        Ok(())
    }

    /// Add multiple binary vectors in batch
    pub fn add_batch_binary(&mut self, ids: &[u64], bits: &[Vec<u8>]) -> Result<()> {
        if ids.len() != bits.len() {
            return Err(CxpError::Search(
                "IDs and vectors length mismatch".to_string(),
            ));
        }

        for (id, b) in ids.iter().zip(bits.iter()) {
            self.add_binary(*id, b)?;
        }

        Ok(())
    }

    /// Search for k nearest neighbors of a float32 vector
    pub fn search_f32(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        if query.len() != self.config.dimensions {
            return Err(CxpError::Search(format!(
                "Query dimension mismatch: expected {}, got {}",
                self.config.dimensions,
                query.len()
            )));
        }

        let results = self
            .index
            .search(query, k)
            .map_err(|e| CxpError::Search(format!("Search failed: {}", e)))?;

        Ok(results
            .keys
            .iter()
            .zip(results.distances.iter())
            .map(|(&id, &distance)| SearchResult { id, distance })
            .collect())
    }

    /// Search for k nearest neighbors of a binary vector
    pub fn search_binary(&self, bits: &[u8], k: usize) -> Result<Vec<SearchResult>> {
        if bits.len() != self.config.dimensions {
            return Err(CxpError::Search(format!(
                "Query binary size mismatch: expected {} bytes, got {}",
                self.config.dimensions,
                bits.len()
            )));
        }

        let results = self
            .index
            .search(bits, k)
            .map_err(|e| CxpError::Search(format!("Search failed: {}", e)))?;

        Ok(results
            .keys
            .iter()
            .zip(results.distances.iter())
            .map(|(&id, &distance)| SearchResult { id, distance })
            .collect())
    }

    /// Search using a BinaryEmbedding
    #[cfg(feature = "embeddings")]
    pub fn search_binary_embedding(
        &self,
        embedding: &BinaryEmbedding,
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        self.search_binary(&embedding.bits, k)
    }

    /// Save index to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| CxpError::Search("Invalid path".to_string()))?;

        self.index
            .save(path_str)
            .map_err(|e| CxpError::Search(format!("Failed to save index: {}", e)))?;

        Ok(())
    }

    /// Load index from disk
    pub fn load<P: AsRef<Path>>(path: P, config: HnswConfig) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| CxpError::Search("Invalid path".to_string()))?;

        let scalar_kind = match config.metric {
            DistanceMetric::Hamming => ScalarKind::B1,
            _ => ScalarKind::F32,
        };

        let options = usearch::IndexOptions {
            dimensions: config.dimensions,
            metric: config.metric.to_usearch_metric(),
            quantization: scalar_kind,
            connectivity: config.connectivity,
            expansion_add: config.expansion_add,
            expansion_search: config.expansion_search,
        };

        let index = Index::new(&options)
            .map_err(|e| CxpError::Search(format!("Failed to create index: {}", e)))?;

        index
            .load(path_str)
            .map_err(|e| CxpError::Search(format!("Failed to load index: {}", e)))?;

        Ok(Self {
            index,
            config,
            scalar_kind,
        })
    }

    /// Get the number of vectors in the index
    pub fn len(&self) -> usize {
        self.index.size()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get index configuration
    pub fn config(&self) -> &HnswConfig {
        &self.config
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.index.clear();
    }

    /// Remove a vector by ID
    pub fn remove(&mut self, id: u64) -> Result<()> {
        self.index
            .remove(id)
            .map_err(|e| CxpError::Search(format!("Failed to remove vector: {}", e)))?;
        Ok(())
    }

    /// Check if a vector exists in the index
    pub fn contains(&self, id: u64) -> bool {
        self.index.contains(id)
    }

    /// Set the search expansion parameter (ef_search)
    /// Higher values = better recall, slower search
    pub fn set_expansion_search(&mut self, expansion: usize) {
        self.index.change_expansion_search(expansion);
    }
}

/// Search result containing ID and distance
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// Vector ID
    pub id: u64,
    /// Distance to query vector (lower is more similar for most metrics)
    pub distance: f32,
}

impl SearchResult {
    /// Convert distance to similarity score (0.0 to 1.0)
    /// Higher is more similar
    pub fn similarity(&self, metric: DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::Cosine => 1.0 - self.distance,
            DistanceMetric::IP => self.distance, // Inner product is already a similarity
            DistanceMetric::L2 | DistanceMetric::Hamming => {
                // Convert distance to similarity (inverse with normalization)
                1.0 / (1.0 + self.distance)
            }
        }
    }
}

#[cfg(all(test, feature = "search"))]
mod tests {
    use super::*;

    #[test]
    fn test_create_index_cosine() {
        let config = HnswConfig::float32_cosine(128);
        let index = HnswIndex::new(config).unwrap();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_create_index_binary() {
        let config = HnswConfig::binary(256);
        let index = HnswIndex::new(config).unwrap();
        assert_eq!(index.config.dimensions, 32); // 256 bits = 32 bytes
    }

    #[test]
    fn test_add_and_search_f32() {
        let config = HnswConfig::float32_cosine(4);
        let mut index = HnswIndex::new(config).unwrap();

        // Add some vectors
        index.add_f32(1, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        index.add_f32(2, &[0.0, 1.0, 0.0, 0.0]).unwrap();
        index.add_f32(3, &[0.0, 0.0, 1.0, 0.0]).unwrap();

        assert_eq!(index.len(), 3);

        // Search for nearest neighbor of [1.0, 0.1, 0.0, 0.0]
        let results = index.search_f32(&[1.0, 0.1, 0.0, 0.0], 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1); // Should be closest to vector 1
    }

    #[test]
    fn test_add_and_search_binary() {
        let config = HnswConfig::binary(8); // 8 bits = 1 byte
        let mut index = HnswIndex::new(config).unwrap();

        // Add binary vectors
        index.add_binary(1, &[0b10101010]).unwrap();
        index.add_binary(2, &[0b11110000]).unwrap();
        index.add_binary(3, &[0b00001111]).unwrap();

        assert_eq!(index.len(), 3);

        // Search for nearest to 0b10101010
        let results = index.search_binary(&[0b10101010], 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_batch_add_f32() {
        let config = HnswConfig::float32_cosine(4);
        let mut index = HnswIndex::new(config).unwrap();

        let ids = vec![1, 2, 3];
        let vectors = vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];

        index.add_batch_f32(&ids, &vectors).unwrap();
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_remove_vector() {
        let config = HnswConfig::float32_cosine(4);
        let mut index = HnswIndex::new(config).unwrap();

        index.add_f32(1, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        index.add_f32(2, &[0.0, 1.0, 0.0, 0.0]).unwrap();

        assert_eq!(index.len(), 2);
        assert!(index.contains(1));

        index.remove(1).unwrap();
        assert_eq!(index.len(), 1);
        assert!(!index.contains(1));
    }

    #[test]
    fn test_clear_index() {
        let config = HnswConfig::float32_cosine(4);
        let mut index = HnswIndex::new(config).unwrap();

        index.add_f32(1, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        index.add_f32(2, &[0.0, 1.0, 0.0, 0.0]).unwrap();

        assert_eq!(index.len(), 2);

        index.clear();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_search_result_similarity() {
        let result = SearchResult {
            id: 1,
            distance: 0.2,
        };

        let cosine_sim = result.similarity(DistanceMetric::Cosine);
        assert!((cosine_sim - 0.8).abs() < 0.001);

        let l2_sim = result.similarity(DistanceMetric::L2);
        assert!(l2_sim > 0.0 && l2_sim < 1.0);
    }

    #[cfg(feature = "embeddings")]
    #[test]
    fn test_binary_embedding_integration() {
        let config = HnswConfig::binary(8);
        let mut index = HnswIndex::new(config).unwrap();

        let embedding = BinaryEmbedding::from_float(&[1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);

        index.add_binary_embedding(1, &embedding).unwrap();
        assert_eq!(index.len(), 1);

        let results = index.search_binary_embedding(&embedding, 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[0].distance, 0.0); // Exact match
    }

    #[test]
    fn test_dimension_mismatch() {
        let config = HnswConfig::float32_cosine(4);
        let mut index = HnswIndex::new(config).unwrap();

        // Try to add vector with wrong dimensions
        let result = index.add_f32(1, &[1.0, 0.0, 0.0]);
        assert!(result.is_err());
    }
}
