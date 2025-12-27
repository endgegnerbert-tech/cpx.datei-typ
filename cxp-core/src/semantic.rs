//! Semantic Embeddings Storage
//!
//! This module provides serialization and deserialization for binary and Int8 embeddings
//! that are stored inside CXP archives.
//!
//! Features:
//! - Binary and Int8 embedding storage
//! - Compact serialization format
//! - Integration with HNSW index

#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
use crate::{BinaryEmbedding, Int8Embedding, CxpError, Result};

#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
use serde::{Deserialize, Serialize};

/// Container for storing embeddings in a CXP archive
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStore {
    /// Binary embeddings (compact representation)
    pub binary: Vec<BinaryEmbedding>,
    /// Int8 embeddings (for rescoring)
    pub int8: Vec<Int8Embedding>,
    /// Original embedding dimensions
    pub dimensions: usize,
}

#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
impl EmbeddingStore {
    /// Create a new embedding store
    pub fn new(binary: Vec<BinaryEmbedding>, int8: Vec<Int8Embedding>, dimensions: usize) -> Self {
        Self {
            binary,
            int8,
            dimensions,
        }
    }

    /// Create from float embeddings
    pub fn from_floats(embeddings: &[Vec<f32>]) -> Self {
        let dimensions = embeddings.first().map(|e| e.len()).unwrap_or(0);
        let binary = embeddings.iter().map(|e| BinaryEmbedding::from_float(e)).collect();
        let int8 = embeddings.iter().map(|e| Int8Embedding::from_float(e)).collect();

        Self {
            binary,
            int8,
            dimensions,
        }
    }

    /// Get the number of embeddings stored
    pub fn len(&self) -> usize {
        self.binary.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.binary.is_empty()
    }

    /// Get total size in bytes
    pub fn size_bytes(&self) -> usize {
        let binary_size: usize = self.binary.iter().map(|e| e.size_bytes()).sum();
        let int8_size: usize = self.int8.iter().map(|e| e.size_bytes()).sum();
        binary_size + int8_size + 8 // +8 for dimensions
    }

    /// Get a binary embedding by index
    pub fn get_binary(&self, index: usize) -> Option<&BinaryEmbedding> {
        self.binary.get(index)
    }

    /// Get an Int8 embedding by index
    pub fn get_int8(&self, index: usize) -> Option<&Int8Embedding> {
        self.int8.get(index)
    }
}

/// Serialize binary embeddings to a compact binary format
///
/// Format:
/// - u32: number of embeddings
/// - u32: dimensions
/// - For each embedding:
///   - bytes: packed bits
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
pub fn serialize_binary_embeddings(embeddings: &[BinaryEmbedding]) -> Result<Vec<u8>> {
    if embeddings.is_empty() {
        return Ok(Vec::new());
    }

    let count = embeddings.len() as u32;
    let dimensions = embeddings[0].dimensions as u32;
    let bytes_per_embedding = (dimensions + 7) / 8;

    let mut data = Vec::with_capacity(8 + embeddings.len() * bytes_per_embedding as usize);

    // Header
    data.extend_from_slice(&count.to_le_bytes());
    data.extend_from_slice(&dimensions.to_le_bytes());

    // Embeddings
    for emb in embeddings {
        if emb.dimensions as u32 != dimensions {
            return Err(CxpError::Serialization(
                "All binary embeddings must have same dimensions".to_string()
            ));
        }
        data.extend_from_slice(&emb.bits);
    }

    Ok(data)
}

/// Deserialize binary embeddings from binary format
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
pub fn deserialize_binary_embeddings(data: &[u8]) -> Result<Vec<BinaryEmbedding>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    if data.len() < 8 {
        return Err(CxpError::Serialization(
            "Invalid binary embeddings data: too short".to_string()
        ));
    }

    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let dimensions = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let bytes_per_embedding = (dimensions + 7) / 8;

    let expected_size = 8 + count * bytes_per_embedding;
    if data.len() != expected_size {
        return Err(CxpError::Serialization(format!(
            "Invalid binary embeddings data: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut embeddings = Vec::with_capacity(count);
    let mut offset = 8;

    for _ in 0..count {
        let bits = data[offset..offset + bytes_per_embedding].to_vec();
        embeddings.push(BinaryEmbedding { bits, dimensions });
        offset += bytes_per_embedding;
    }

    Ok(embeddings)
}

/// Serialize Int8 embeddings to a compact binary format
///
/// Format:
/// - u32: number of embeddings
/// - u32: dimensions
/// - For each embedding:
///   - f32: scale factor
///   - i8 * dimensions: quantized values
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
pub fn serialize_int8_embeddings(embeddings: &[Int8Embedding]) -> Result<Vec<u8>> {
    if embeddings.is_empty() {
        return Ok(Vec::new());
    }

    let count = embeddings.len() as u32;
    let dimensions = embeddings[0].values.len() as u32;

    let mut data = Vec::with_capacity(8 + embeddings.len() * (4 + dimensions as usize));

    // Header
    data.extend_from_slice(&count.to_le_bytes());
    data.extend_from_slice(&dimensions.to_le_bytes());

    // Embeddings
    for emb in embeddings {
        if emb.values.len() as u32 != dimensions {
            return Err(CxpError::Serialization(
                "All Int8 embeddings must have same dimensions".to_string()
            ));
        }

        // Write scale
        data.extend_from_slice(&emb.scale.to_le_bytes());

        // Write values (i8 can be written directly as bytes)
        for &val in &emb.values {
            data.push(val as u8);
        }
    }

    Ok(data)
}

/// Deserialize Int8 embeddings from binary format
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
pub fn deserialize_int8_embeddings(data: &[u8]) -> Result<Vec<Int8Embedding>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    if data.len() < 8 {
        return Err(CxpError::Serialization(
            "Invalid Int8 embeddings data: too short".to_string()
        ));
    }

    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let dimensions = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let bytes_per_embedding = 4 + dimensions; // 4 bytes for scale + dimensions bytes for values

    let expected_size = 8 + count * bytes_per_embedding;
    if data.len() != expected_size {
        return Err(CxpError::Serialization(format!(
            "Invalid Int8 embeddings data: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut embeddings = Vec::with_capacity(count);
    let mut offset = 8;

    for _ in 0..count {
        // Read scale
        let scale = f32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Read values
        let values: Vec<i8> = data[offset..offset + dimensions]
            .iter()
            .map(|&b| b as i8)
            .collect();
        offset += dimensions;

        embeddings.push(Int8Embedding { values, scale });
    }

    Ok(embeddings)
}

#[cfg(test)]
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_store_creation() {
        let embeddings = vec![
            vec![0.1, 0.2, 0.3, 0.4],
            vec![0.5, 0.6, 0.7, 0.8],
        ];

        let store = EmbeddingStore::from_floats(&embeddings);

        assert_eq!(store.len(), 2);
        assert_eq!(store.dimensions, 4);
        assert!(!store.is_empty());
    }

    #[test]
    fn test_binary_serialization_roundtrip() {
        let embeddings = vec![
            vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0],
            vec![-1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0],
        ];

        let binary: Vec<_> = embeddings
            .iter()
            .map(|e| BinaryEmbedding::from_float(e))
            .collect();

        let serialized = serialize_binary_embeddings(&binary).unwrap();
        let deserialized = deserialize_binary_embeddings(&serialized).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].dimensions, 8);
        assert_eq!(deserialized[0].bits, binary[0].bits);
        assert_eq!(deserialized[1].bits, binary[1].bits);
    }

    #[test]
    fn test_int8_serialization_roundtrip() {
        let embeddings = vec![
            vec![1.0, 0.5, -0.5, -1.0],
            vec![0.8, 0.3, -0.3, -0.8],
        ];

        let int8: Vec<_> = embeddings
            .iter()
            .map(|e| Int8Embedding::from_float(e))
            .collect();

        let serialized = serialize_int8_embeddings(&int8).unwrap();
        let deserialized = deserialize_int8_embeddings(&serialized).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].values.len(), 4);

        // Check that scale factors are preserved
        assert!((deserialized[0].scale - int8[0].scale).abs() < 0.001);
        assert!((deserialized[1].scale - int8[1].scale).abs() < 0.001);
    }

    #[test]
    fn test_empty_embeddings() {
        let empty_binary: Vec<BinaryEmbedding> = Vec::new();
        let serialized = serialize_binary_embeddings(&empty_binary).unwrap();
        assert!(serialized.is_empty());

        let deserialized = deserialize_binary_embeddings(&serialized).unwrap();
        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_dimension_mismatch_binary() {
        let emb1 = BinaryEmbedding::from_float(&[1.0, -1.0, 1.0, -1.0]);
        let emb2 = BinaryEmbedding::from_float(&[1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);

        let result = serialize_binary_embeddings(&[emb1, emb2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_dimension_mismatch_int8() {
        let emb1 = Int8Embedding::from_float(&[1.0, 0.5, -0.5, -1.0]);
        let emb2 = Int8Embedding::from_float(&[1.0, 0.5]);

        let result = serialize_int8_embeddings(&[emb1, emb2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_store_size_calculation() {
        let embeddings = vec![
            vec![0.0f32; 384], // MiniLM dimensions
            vec![0.0f32; 384],
        ];

        let store = EmbeddingStore::from_floats(&embeddings);

        // Binary: 384 bits = 48 bytes per embedding = 96 bytes total
        // Int8: 384 + 4 bytes per embedding = 776 bytes total
        // Total: 96 + 776 + 8 = 880 bytes
        assert_eq!(store.size_bytes(), 880);
    }
}
