//! Multimodal embedding generation using SigLIP 2 with ONNX Runtime
//!
//! SigLIP 2 provides a unified embedding space for both images and text:
//! - Image embeddings: 512 dimensions
//! - Text embeddings: 512 dimensions
//! - Same vector space enables cross-modal search
//!
//! Features:
//! - Image preprocessing (resize, normalize)
//! - Batch processing for images
//! - Text embeddings via SigLIP 2 text encoder
//! - Binary and Int8 quantization support

use crate::{CxpError, Result};

use ndarray::{Array3, Array4, Array2};
use ort::session::Session;
use ort::value::Value;
use std::path::Path;

// Re-export binary and int8 embedding types from embeddings module
// These are always available when multimodal feature is enabled
pub use crate::embeddings::{BinaryEmbedding, Int8Embedding};

/// SigLIP 2 image preprocessing constants
const IMAGE_SIZE: u32 = 224;
const NORMALIZE_MEAN: [f32; 3] = [0.5, 0.5, 0.5];
const NORMALIZE_STD: [f32; 3] = [0.5, 0.5, 0.5];

/// Embedding dimension for SigLIP 2
pub const SIGLIP2_DIMENSIONS: usize = 512;

/// Multimodal embedding engine using SigLIP 2
///
/// SigLIP 2 creates embeddings in a shared vector space, allowing
/// semantic search across images and text.
///
/// # Example
/// ```no_run
/// use cxp_core::multimodal::MultimodalEngine;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let engine = MultimodalEngine::load("models/siglip2")?;
///
/// // Embed an image
/// let img_embedding = engine.embed_image(Path::new("photo.jpg"))?;
///
/// // Embed text
/// let text_embedding = engine.embed_text("a beautiful sunset")?;
///
/// // Compare similarity (cosine similarity via dot product of normalized vectors)
/// let similarity = cosine_similarity(&img_embedding, &text_embedding);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "multimodal")]
pub struct MultimodalEngine {
    /// ONNX session for image encoder
    image_session: Session,
    /// ONNX session for text encoder
    text_session: Session,
    /// Tokenizer for text processing
    tokenizer: tokenizers::Tokenizer,
    /// Maximum sequence length for text
    max_length: usize,
}

#[cfg(feature = "multimodal")]
impl MultimodalEngine {
    /// Load SigLIP 2 model from a directory
    ///
    /// Expected directory structure:
    /// ```text
    /// model_dir/
    ///   image_encoder.onnx
    ///   text_encoder.onnx
    ///   tokenizer.json
    /// ```
    pub fn load<P: AsRef<Path>>(model_dir: P) -> Result<Self> {
        let model_dir = model_dir.as_ref();

        // Load image encoder
        let image_path = model_dir.join("image_encoder.onnx");
        let image_session = Session::builder()?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get())?
            .commit_from_file(&image_path)
            .map_err(|e| CxpError::Embedding(format!("Failed to load image encoder: {}", e)))?;

        // Load text encoder
        let text_path = model_dir.join("text_encoder.onnx");
        let text_session = Session::builder()?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get())?
            .commit_from_file(&text_path)
            .map_err(|e| CxpError::Embedding(format!("Failed to load text encoder: {}", e)))?;

        // Load tokenizer
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| CxpError::Embedding(format!("Failed to load tokenizer: {}", e)))?;

        Ok(Self {
            image_session,
            text_session,
            tokenizer,
            max_length: 64, // SigLIP 2 typically uses shorter sequences
        })
    }

    /// Embed a single image from file path
    ///
    /// Returns a 512-dimensional embedding vector.
    pub fn embed_image(&mut self, image_path: &Path) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch_images(&[image_path])?;
        embeddings.into_iter().next()
            .ok_or_else(|| CxpError::Embedding("No embedding generated".into()))
    }

    /// Embed multiple images in a batch
    ///
    /// More efficient than calling `embed_image` multiple times.
    pub fn embed_batch_images(&mut self, image_paths: &[&Path]) -> Result<Vec<Vec<f32>>> {
        if image_paths.is_empty() {
            return Ok(Vec::new());
        }

        // Load and preprocess all images
        let batch_size = image_paths.len();
        let mut batch_tensor = Array4::<f32>::zeros((batch_size, 3, IMAGE_SIZE as usize, IMAGE_SIZE as usize));

        for (i, path) in image_paths.iter().enumerate() {
            let img = image::open(path)
                .map_err(|e| CxpError::Embedding(format!("Failed to open image {}: {}", path.display(), e)))?;

            let preprocessed = preprocess_image(img)?;
            batch_tensor.slice_mut(ndarray::s![i, .., .., ..]).assign(&preprocessed);
        }

        // Run inference
        let pixel_values = Value::from_array(batch_tensor)?;
        let outputs = self.image_session.run(ort::inputs![
            "pixel_values" => pixel_values,
        ])?;

        // Extract embeddings
        let embeddings = outputs["image_embeds"]
            .try_extract_array::<f32>()?
            .into_dimensionality::<ndarray::Ix2>()
            .map_err(|e| CxpError::Embedding(format!("Failed to convert to 2D array: {}", e)))?
            .to_owned();

        // Normalize embeddings (L2 normalization)
        let normalized = normalize_embeddings(embeddings)?;

        Ok(normalized.outer_iter().map(|row| row.to_vec()).collect())
    }

    /// Embed text using SigLIP 2 text encoder
    ///
    /// Returns a 512-dimensional embedding in the same space as images.
    pub fn embed_text(&mut self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch_text(&[text])?;
        batch.into_iter().next()
            .ok_or_else(|| CxpError::Embedding("No embedding generated".into()))
    }

    /// Embed multiple texts in a batch
    pub fn embed_batch_text(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
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
        let input_ids_value = Value::from_array(input_ids)?;
        let attention_mask_value = Value::from_array(attention_mask)?;
        let outputs = self.text_session.run(ort::inputs![
            "input_ids" => input_ids_value,
            "attention_mask" => attention_mask_value,
        ])?;

        // Extract embeddings
        let embeddings = outputs["text_embeds"]
            .try_extract_array::<f32>()?
            .into_dimensionality::<ndarray::Ix2>()
            .map_err(|e| CxpError::Embedding(format!("Failed to convert to 2D array: {}", e)))?
            .to_owned();

        // Normalize embeddings
        let normalized = normalize_embeddings(embeddings)?;

        Ok(normalized.outer_iter().map(|row| row.to_vec()).collect())
    }

    /// Generate binary embeddings for images
    pub fn embed_image_binary(&mut self, image_path: &Path) -> Result<BinaryEmbedding> {
        let embedding = self.embed_image(image_path)?;
        Ok(BinaryEmbedding::from_float(&embedding))
    }

    /// Generate Int8 embeddings for images
    pub fn embed_image_int8(&mut self, image_path: &Path) -> Result<Int8Embedding> {
        let embedding = self.embed_image(image_path)?;
        Ok(Int8Embedding::from_float(&embedding))
    }

    /// Generate binary embeddings for text
    pub fn embed_text_binary(&mut self, text: &str) -> Result<BinaryEmbedding> {
        let embedding = self.embed_text(text)?;
        Ok(BinaryEmbedding::from_float(&embedding))
    }

    /// Generate Int8 embeddings for text
    pub fn embed_text_int8(&mut self, text: &str) -> Result<Int8Embedding> {
        let embedding = self.embed_text(text)?;
        Ok(Int8Embedding::from_float(&embedding))
    }

    /// Get embedding dimensions (always 512 for SigLIP 2)
    pub fn dimensions(&self) -> usize {
        SIGLIP2_DIMENSIONS
    }
}

/// Preprocess an image for SigLIP 2
///
/// Steps:
/// 1. Resize to 224x224
/// 2. Convert to RGB
/// 3. Normalize with mean=[0.5, 0.5, 0.5] and std=[0.5, 0.5, 0.5]
/// 4. Convert to CHW format (Channel, Height, Width)
fn preprocess_image(img: image::DynamicImage) -> Result<Array3<f32>> {
    // Resize to 224x224
    let img = img.resize_exact(
        IMAGE_SIZE,
        IMAGE_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    // Convert to RGB
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    // Create CHW tensor
    let mut tensor = Array3::<f32>::zeros((3, height as usize, width as usize));

    // Fill tensor and normalize
    for y in 0..height {
        for x in 0..width {
            let pixel = rgb.get_pixel(x, y);
            for c in 0..3 {
                let value = pixel[c] as f32 / 255.0; // Convert to [0, 1]
                let normalized = (value - NORMALIZE_MEAN[c]) / NORMALIZE_STD[c];
                tensor[[c, y as usize, x as usize]] = normalized;
            }
        }
    }

    Ok(tensor)
}

/// L2 normalize embeddings (converts to unit vectors)
///
/// This is required for SigLIP 2 embeddings to enable cosine similarity
/// computation via simple dot product.
fn normalize_embeddings(embeddings: Array2<f32>) -> Result<Array2<f32>> {
    let mut normalized = embeddings.clone();

    for mut row in normalized.outer_iter_mut() {
        let norm: f32 = row.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-12 {
            row /= norm;
        }
    }

    Ok(normalized)
}

/// Compute cosine similarity between two normalized embeddings
///
/// For normalized vectors, this is simply the dot product.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Embeddings must have same dimension");
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Compute cosine distance (1 - similarity)
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    1.0 - cosine_similarity(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_embeddings() {
        let embeddings = Array2::<f32>::from_shape_vec(
            (2, 4),
            vec![
                3.0, 4.0, 0.0, 0.0, // norm = 5
                1.0, 0.0, 0.0, 0.0, // norm = 1
            ],
        ).unwrap();

        let normalized = normalize_embeddings(embeddings).unwrap();

        // First row should be [0.6, 0.8, 0.0, 0.0]
        assert!((normalized[[0, 0]] - 0.6).abs() < 1e-6);
        assert!((normalized[[0, 1]] - 0.8).abs() < 1e-6);

        // Second row should be [1.0, 0.0, 0.0, 0.0]
        assert!((normalized[[1, 0]] - 1.0).abs() < 1e-6);

        // Check that all rows have unit norm
        for row in normalized.outer_iter() {
            let norm: f32 = row.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.6, 0.8, 0.0, 0.0];
        let d = vec![0.8, 0.6, 0.0, 0.0];
        let sim = cosine_similarity(&c, &d);
        // 0.6*0.8 + 0.8*0.6 = 0.96
        assert!((sim - 0.96).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        assert!(cosine_distance(&a, &b).abs() < 1e-6);

        let c = vec![1.0, 0.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0, 0.0];
        assert!((cosine_distance(&c, &d) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_preprocess_image() {
        // Create a simple 4x4 red image
        let img = image::DynamicImage::ImageRgb8(
            image::RgbImage::from_pixel(4, 4, image::Rgb([255, 0, 0]))
        );

        let tensor = preprocess_image(img).unwrap();

        // Check shape: 3 channels x 224 x 224
        assert_eq!(tensor.shape(), &[3, IMAGE_SIZE as usize, IMAGE_SIZE as usize]);

        // Red channel should be (1.0 - 0.5) / 0.5 = 1.0
        assert!((tensor[[0, 0, 0]] - 1.0).abs() < 1e-6);

        // Green and blue channels should be (0.0 - 0.5) / 0.5 = -1.0
        assert!((tensor[[1, 0, 0]] + 1.0).abs() < 1e-6);
        assert!((tensor[[2, 0, 0]] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dimensions_constant() {
        assert_eq!(SIGLIP2_DIMENSIONS, 512);
    }
}
