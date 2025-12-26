//! Embedding generation using tract-onnx (WASM-compatible)
//!
//! This module provides a WASM-compatible alternative to the ort-based embedding engine.
//! It uses tract-onnx for pure Rust ONNX inference without platform-specific dependencies.

#[cfg(feature = "embeddings-wasm")]
use crate::{CxpError, Result};

#[cfg(feature = "embeddings-wasm")]
use crate::embeddings::{BinaryEmbedding, Int8Embedding, EmbeddingModel};

#[cfg(feature = "embeddings-wasm")]
use ndarray::{s, Array2, Axis};
#[cfg(feature = "embeddings-wasm")]
use tokenizers::Tokenizer;
#[cfg(feature = "embeddings-wasm")]
use tract_onnx::prelude::*;

#[cfg(feature = "embeddings-wasm")]
use std::path::Path;

/// Tract-based embedding engine (WASM-compatible)
#[cfg(feature = "embeddings-wasm")]
pub struct TractEmbeddingEngine {
    /// Tract model
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    /// Tokenizer
    tokenizer: Tokenizer,
    /// Model type
    embedding_model: EmbeddingModel,
    /// Maximum sequence length
    max_length: usize,
}

#[cfg(feature = "embeddings-wasm")]
impl TractEmbeddingEngine {
    /// Load an embedding model from a directory
    pub fn load<P: AsRef<Path>>(model_dir: P, model: EmbeddingModel) -> Result<Self> {
        let model_dir = model_dir.as_ref();

        // Load ONNX model with tract
        let model_path = model_dir.join("model.onnx");
        let tract_model = tract_onnx::onnx()
            .model_for_path(&model_path)
            .map_err(|e| CxpError::Embedding(format!("Failed to load ONNX model: {}", e)))?
            .into_optimized()
            .map_err(|e| CxpError::Embedding(format!("Failed to optimize model: {}", e)))?
            .into_runnable()
            .map_err(|e| CxpError::Embedding(format!("Failed to create runnable model: {}", e)))?;

        // Load tokenizer
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| CxpError::Embedding(format!("Failed to load tokenizer: {}", e)))?;

        Ok(Self {
            model: tract_model,
            tokenizer,
            embedding_model: model,
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

        // Convert to tract tensors
        let input_ids_tensor = tract_ndarray::Array2::from_shape_vec(
            (batch_size, seq_len),
            input_ids.iter().copied().collect(),
        )
        .map_err(|e| CxpError::Embedding(format!("Failed to create input_ids tensor: {}", e)))?
        .into_dyn();

        let attention_mask_tensor = tract_ndarray::Array2::from_shape_vec(
            (batch_size, seq_len),
            attention_mask.iter().copied().collect(),
        )
        .map_err(|e| CxpError::Embedding(format!("Failed to create attention_mask tensor: {}", e)))?
        .into_dyn();

        // Run inference
        let outputs = self.model
            .run(tvec![
                input_ids_tensor.into(),
                attention_mask_tensor.into()
            ])
            .map_err(|e| CxpError::Embedding(format!("Inference failed: {}", e)))?;

        // Extract embeddings from output
        // The output is typically the first tensor
        let output_tensor = outputs
            .get(0)
            .ok_or_else(|| CxpError::Embedding("No output tensor found".into()))?;

        let output_array = output_tensor
            .to_array_view::<f32>()
            .map_err(|e| CxpError::Embedding(format!("Failed to convert output to array: {}", e)))?;

        // Handle different output shapes
        let result = match output_array.shape() {
            // Shape: [batch_size, embedding_dim] - already pooled
            [b, d] if *b == batch_size && *d == self.embedding_model.dimensions() => {
                (0..batch_size)
                    .map(|i| {
                        output_array
                            .slice(s![i, ..])
                            .iter()
                            .copied()
                            .collect::<Vec<f32>>()
                    })
                    .collect()
            }
            // Shape: [batch_size, seq_len, hidden_dim] - needs mean pooling
            [b, _s, d] if *b == batch_size && *d == self.embedding_model.dimensions() => {
                (0..batch_size)
                    .map(|i| {
                        let sequence = output_array.slice(s![i, .., ..]);
                        let seq_len_actual = attention_mask.row(i).iter().filter(|&&x| x != 0).count();

                        // Mean pooling over sequence dimension
                        let mut pooled = vec![0.0f32; *d];
                        for t in 0..seq_len_actual {
                            for dim in 0..*d {
                                pooled[dim] += sequence[[t, dim]];
                            }
                        }

                        if seq_len_actual > 0 {
                            for val in pooled.iter_mut() {
                                *val /= seq_len_actual as f32;
                            }
                        }

                        pooled
                    })
                    .collect()
            }
            shape => {
                return Err(CxpError::Embedding(format!(
                    "Unexpected output shape: {:?}, expected [batch_size, {}] or [batch_size, seq_len, {}]",
                    shape,
                    self.embedding_model.dimensions(),
                    self.embedding_model.dimensions()
                )));
            }
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
        self.embedding_model
    }

    /// Get the embedding dimensions
    pub fn dimensions(&self) -> usize {
        self.embedding_model.dimensions()
    }
}

#[cfg(test)]
#[cfg(feature = "embeddings-wasm")]
mod tests {
    use super::*;

    #[test]
    fn test_tract_engine_api() {
        // This test just ensures the API is consistent
        // Actual model testing would require loading a real model

        // Just verify that the types and methods exist
        fn _verify_api(engine: &TractEmbeddingEngine) {
            let _ = engine.model();
            let _ = engine.dimensions();
        }
    }
}
