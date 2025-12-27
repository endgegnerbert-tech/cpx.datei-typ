//! Embedding generation using ONNX Runtime
//!
//! Supports multiple embedding models:
//! - all-MiniLM-L6-v2 (384 dims, 90MB)
//! - EmbeddingGemma (768 dims with MRL, 200MB)
//!
//! Features:
//! - Binary quantization (32x smaller vectors)
//! - Int8 quantization for rescoring
//! - Batch processing

// Core types are available with either embeddings or embeddings-wasm
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
use crate::{CxpError, Result};

// ONNX-specific imports (only for native embeddings)
#[cfg(feature = "embeddings")]
use ndarray::{Array2, Axis};
#[cfg(feature = "embeddings")]
use ort::session::Session;
#[cfg(feature = "embeddings")]
use tokenizers::Tokenizer;

#[cfg(feature = "embeddings")]
use std::path::Path;

/// Supported embedding models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModel {
    /// all-MiniLM-L6-v2 - 384 dimensions, 90MB
    MiniLM,
    /// EmbeddingGemma - 768 dimensions (MRL: 512/256/128), 200MB
    EmbeddingGemma,
}

impl EmbeddingModel {
    /// Get the embedding dimension for this model
    pub fn dimensions(&self) -> usize {
        match self {
            EmbeddingModel::MiniLM => 384,
            EmbeddingModel::EmbeddingGemma => 768,
        }
    }

    /// Get the model name
    pub fn name(&self) -> &'static str {
        match self {
            EmbeddingModel::MiniLM => "all-MiniLM-L6-v2",
            EmbeddingModel::EmbeddingGemma => "EmbeddingGemma",
        }
    }
}

/// Binary embedding (32x smaller than float32)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryEmbedding {
    /// Packed bits - each bit represents sign of original value
    pub bits: Vec<u8>,
    /// Original dimension count
    pub dimensions: usize,
}

impl BinaryEmbedding {
    /// Create from float32 embedding using sign-based quantization
    pub fn from_float(embedding: &[f32]) -> Self {
        let dimensions = embedding.len();
        let byte_count = (dimensions + 7) / 8;
        let mut bits = vec![0u8; byte_count];

        for (i, &value) in embedding.iter().enumerate() {
            if value > 0.0 {
                bits[i / 8] |= 1 << (i % 8);
            }
        }

        Self { bits, dimensions }
    }

    /// Compute Hamming distance to another binary embedding
    pub fn hamming_distance(&self, other: &BinaryEmbedding) -> u32 {
        self.bits
            .iter()
            .zip(other.bits.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }

    /// Size in bytes
    pub fn size_bytes(&self) -> usize {
        self.bits.len()
    }
}

/// Int8 embedding (4x smaller than float32)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Int8Embedding {
    /// Quantized values (-128 to 127)
    pub values: Vec<i8>,
    /// Scale factor for dequantization
    pub scale: f32,
}

impl Int8Embedding {
    /// Create from float32 embedding using linear quantization
    pub fn from_float(embedding: &[f32]) -> Self {
        // Find the max absolute value
        let max_abs = embedding
            .iter()
            .map(|x| x.abs())
            .fold(0.0f32, f32::max);

        let scale = if max_abs > 0.0 { max_abs / 127.0 } else { 1.0 };

        let values: Vec<i8> = embedding
            .iter()
            .map(|&x| (x / scale).round().clamp(-128.0, 127.0) as i8)
            .collect();

        Self { values, scale }
    }

    /// Compute dot product with another Int8 embedding (returns approximate score)
    pub fn dot_product(&self, other: &Int8Embedding) -> f32 {
        let sum: i32 = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(&a, &b)| (a as i32) * (b as i32))
            .sum();

        sum as f32 * self.scale * other.scale
    }

    /// Size in bytes
    pub fn size_bytes(&self) -> usize {
        self.values.len() + 4 // values + scale
    }
}

/// Embedding engine for generating embeddings using ONNX
#[cfg(feature = "embeddings")]
pub struct EmbeddingEngine {
    /// ONNX session
    session: Session,
    /// Tokenizer
    tokenizer: Tokenizer,
    /// Model type
    model: EmbeddingModel,
    /// Maximum sequence length
    max_length: usize,
}

#[cfg(feature = "embeddings")]
impl EmbeddingEngine {
    /// Load an embedding model from a directory
    pub fn load<P: AsRef<Path>>(model_dir: P, model: EmbeddingModel) -> Result<Self> {
        let model_dir = model_dir.as_ref();

        // Load ONNX model
        let model_path = model_dir.join("model.onnx");
        let session = Session::builder()?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get())?
            .commit_from_file(&model_path)?;

        // Load tokenizer
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| CxpError::Embedding(format!("Failed to load tokenizer: {}", e)))?;

        Ok(Self {
            session,
            tokenizer,
            model,
            max_length: 512,
        })
    }

    /// Generate embeddings for a batch of texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Tokenize
        let encodings = self.tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| CxpError::Embedding(format!("Tokenization failed: {}", e)))?;

        // Prepare input tensors
        let batch_size = encodings.len();
        let seq_len = encodings.iter().map(|e| e.len()).max().unwrap_or(0).min(self.max_length);

        let mut input_ids = Array2::<i64>::zeros((batch_size, seq_len));
        let mut attention_mask = Array2::<i64>::zeros((batch_size, seq_len));

        for (i, encoding) in encodings.iter().enumerate() {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();
            let len = ids.len().min(seq_len);

            for j in 0..len {
                input_ids[[i, j]] = ids[j] as i64;
                attention_mask[[i, j]] = mask[j] as i64;
            }
        }

        // Run inference
        let outputs = self.session.run(ort::inputs! {
            "input_ids" => input_ids.view(),
            "attention_mask" => attention_mask.view(),
        }?)?;

        // Extract embeddings (usually last hidden state with mean pooling)
        let output = outputs.get("last_hidden_state")
            .or_else(|| outputs.get("sentence_embedding"))
            .ok_or_else(|| CxpError::Embedding("No embedding output found".into()))?;

        let embeddings: Array2<f32> = output.extract_tensor::<f32>()?.to_owned();

        // Mean pooling if needed
        let result: Vec<Vec<f32>> = if embeddings.shape()[1] == self.model.dimensions() {
            // Already pooled
            embeddings.outer_iter().map(|row| row.to_vec()).collect()
        } else {
            // Need to apply mean pooling
            embeddings
                .mean_axis(Axis(1))
                .ok_or_else(|| CxpError::Embedding("Mean pooling failed".into()))?
                .outer_iter()
                .map(|row| row.to_vec())
                .collect()
        };

        Ok(result)
    }

    /// Generate embedding for a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text])?;
        batch.into_iter().next()
            .ok_or_else(|| CxpError::Embedding("No embedding generated".into()))
    }

    /// Generate binary embeddings for a batch
    pub fn embed_binary_batch(&self, texts: &[&str]) -> Result<Vec<BinaryEmbedding>> {
        let embeddings = self.embed_batch(texts)?;
        Ok(embeddings.iter().map(|e| BinaryEmbedding::from_float(e)).collect())
    }

    /// Generate Int8 embeddings for a batch
    pub fn embed_int8_batch(&self, texts: &[&str]) -> Result<Vec<Int8Embedding>> {
        let embeddings = self.embed_batch(texts)?;
        Ok(embeddings.iter().map(|e| Int8Embedding::from_float(e)).collect())
    }

    /// Get the model type
    pub fn model(&self) -> EmbeddingModel {
        self.model
    }

    /// Get the embedding dimensions
    pub fn dimensions(&self) -> usize {
        self.model.dimensions()
    }
}

/// Batch embedding results with both binary and int8 representations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuantizedEmbeddings {
    /// Binary embeddings for fast initial search
    pub binary: Vec<BinaryEmbedding>,
    /// Int8 embeddings for rescoring
    pub int8: Vec<Int8Embedding>,
}

impl QuantizedEmbeddings {
    /// Create from float embeddings
    pub fn from_floats(embeddings: &[Vec<f32>]) -> Self {
        let binary = embeddings.iter().map(|e| BinaryEmbedding::from_float(e)).collect();
        let int8 = embeddings.iter().map(|e| Int8Embedding::from_float(e)).collect();
        Self { binary, int8 }
    }

    /// Total size in bytes
    pub fn size_bytes(&self) -> usize {
        let binary_size: usize = self.binary.iter().map(|e| e.size_bytes()).sum();
        let int8_size: usize = self.int8.iter().map(|e| e.size_bytes()).sum();
        binary_size + int8_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_quantization() {
        let embedding = vec![0.5, -0.3, 0.1, -0.8, 0.0, 0.2, -0.1, 0.9];
        let binary = BinaryEmbedding::from_float(&embedding);

        assert_eq!(binary.dimensions, 8);
        assert_eq!(binary.bits.len(), 1);
        // Expected bits: 0=1, 1=0, 2=1, 3=0, 4=0, 5=1, 6=0, 7=1 = 0b10100101 = 0xA5
        assert_eq!(binary.bits[0], 0b10100101);
    }

    #[test]
    fn test_binary_hamming_distance() {
        let emb1 = BinaryEmbedding::from_float(&[1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);
        let emb2 = BinaryEmbedding::from_float(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);

        let distance = emb1.hamming_distance(&emb2);
        assert_eq!(distance, 4); // 4 bits differ
    }

    #[test]
    fn test_int8_quantization() {
        let embedding = vec![0.5, -0.5, 1.0, -1.0];
        let int8 = Int8Embedding::from_float(&embedding);

        assert_eq!(int8.values.len(), 4);
        assert_eq!(int8.values[2], 127); // max positive
        assert_eq!(int8.values[3], -127); // max negative (not -128 due to rounding)
    }

    #[test]
    fn test_int8_dot_product() {
        let emb1 = Int8Embedding::from_float(&[1.0, 0.0, 0.5, -0.5]);
        let emb2 = Int8Embedding::from_float(&[1.0, 0.5, 0.5, 0.5]);

        let dot = emb1.dot_product(&emb2);
        // Should be approximately: 1*1 + 0*0.5 + 0.5*0.5 + (-0.5)*0.5 = 1.0
        assert!((dot - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_quantized_embeddings_size() {
        let embeddings = vec![
            vec![0.0f32; 384], // MiniLM dimensions
            vec![0.0f32; 384],
        ];

        let quantized = QuantizedEmbeddings::from_floats(&embeddings);

        // Binary: 384 bits = 48 bytes per embedding
        // Int8: 384 + 4 bytes per embedding
        let expected_binary = 48 * 2;
        let expected_int8 = (384 + 4) * 2;

        assert_eq!(quantized.size_bytes(), expected_binary + expected_int8);
    }
}
