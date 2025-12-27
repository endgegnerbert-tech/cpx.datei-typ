# SigLIP 2 Multimodal Embeddings

This module provides SigLIP 2 ONNX support for generating embeddings from both images and text in a unified 512-dimensional vector space.

## Features

- **Unified Embedding Space**: Both images and text are embedded into the same 512-dimensional space, enabling cross-modal semantic search
- **Image Processing**: Automatic preprocessing (resize to 224x224, normalization)
- **Batch Processing**: Efficient batch processing for both images and text
- **Quantization Support**: Binary and Int8 quantization for storage efficiency

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cxp-core = { version = "0.1.0", features = ["multimodal"] }
```

## Model Setup

You need to provide SigLIP 2 ONNX models in the following structure:

```
models/siglip2/
├── image_encoder.onnx
├── text_encoder.onnx
└── tokenizer.json
```

### Converting SigLIP 2 to ONNX

You can convert SigLIP 2 from HuggingFace to ONNX using:

```python
from transformers import AutoModel, AutoTokenizer
import torch.onnx

# Load model
model = AutoModel.from_pretrained("google/siglip-base-patch16-224")
tokenizer = AutoTokenizer.from_pretrained("google/siglip-base-patch16-224")

# Export image encoder
# ... (export logic)

# Export text encoder
# ... (export logic)

# Save tokenizer
tokenizer.save_pretrained("models/siglip2/")
```

## Usage

### Basic Example

```rust
use cxp_core::multimodal::MultimodalEngine;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the model
    let mut engine = MultimodalEngine::load("models/siglip2")?;

    // Embed an image
    let image_embedding = engine.embed_image(Path::new("photo.jpg"))?;
    println!("Image embedding dimension: {}", image_embedding.len());

    // Embed text
    let text_embedding = engine.embed_text("a beautiful sunset over the ocean")?;
    println!("Text embedding dimension: {}", text_embedding.len());

    // Compute similarity (cosine similarity via dot product of normalized vectors)
    let similarity = cxp_core::multimodal::cosine_similarity(
        &image_embedding,
        &text_embedding
    );
    println!("Similarity: {:.4}", similarity);

    Ok(())
}
```

### Batch Processing

```rust
use cxp_core::multimodal::MultimodalEngine;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = MultimodalEngine::load("models/siglip2")?;

    // Batch embed images
    let image_paths = [
        Path::new("img1.jpg"),
        Path::new("img2.jpg"),
        Path::new("img3.jpg"),
    ];
    let image_embeddings = engine.embed_batch_images(&image_paths)?;

    // Batch embed texts
    let texts = ["cat", "dog", "bird"];
    let text_embeddings = engine.embed_batch_text(&texts)?;

    Ok(())
}
```

### With Quantization

```rust
use cxp_core::multimodal::MultimodalEngine;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = MultimodalEngine::load("models/siglip2")?;

    // Generate binary embedding (32x smaller)
    let binary = engine.embed_image_binary(Path::new("photo.jpg"))?;
    println!("Binary embedding size: {} bytes", binary.size_bytes());

    // Generate Int8 embedding (4x smaller)
    let int8 = engine.embed_image_int8(Path::new("photo.jpg"))?;
    println!("Int8 embedding size: {} bytes", int8.size_bytes());

    Ok(())
}
```

### Cross-Modal Search

```rust
use cxp_core::multimodal::{MultimodalEngine, cosine_similarity};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = MultimodalEngine::load("models/siglip2")?;

    // Database of images
    let image_paths = [
        Path::new("beach.jpg"),
        Path::new("mountain.jpg"),
        Path::new("city.jpg"),
    ];

    let image_embeddings = engine.embed_batch_images(&image_paths)?;

    // Search with text query
    let query = "scenic ocean view";
    let query_embedding = engine.embed_text(query)?;

    // Find most similar image
    let mut best_match = 0;
    let mut best_score = -1.0;

    for (i, img_emb) in image_embeddings.iter().enumerate() {
        let score = cosine_similarity(&query_embedding, img_emb);
        if score > best_score {
            best_score = score;
            best_match = i;
        }
    }

    println!("Best match: {} (score: {:.4})",
             image_paths[best_match].display(),
             best_score);

    Ok(())
}
```

## Image Preprocessing

Images are automatically preprocessed with the following steps:

1. **Resize**: Images are resized to 224x224 using Lanczos3 filtering
2. **RGB Conversion**: Images are converted to RGB format
3. **Normalization**:
   - Each pixel value is divided by 255 to get [0, 1] range
   - Normalized with mean=[0.5, 0.5, 0.5] and std=[0.5, 0.5, 0.5]
   - Formula: `(value - mean) / std`
4. **CHW Format**: Converted to Channel-Height-Width format for ONNX

## Supported Image Formats

The `image` crate is configured with PNG and JPEG support. Supported formats include:

- JPEG/JPG
- PNG

To add more formats, update the feature flags in `Cargo.toml`:

```toml
image = { version = "0.25", features = ["png", "jpeg", "webp", "tiff"] }
```

## Performance Tips

1. **Use Batch Processing**: Batch processing is significantly faster than processing items one-by-one
2. **Quantization**: Use binary embeddings for initial filtering, then Int8 or full precision for rescoring
3. **Model Optimization**: Ensure ONNX models are optimized (quantized, pruned) for best performance

## Embedding Dimensions

SigLIP 2 produces **512-dimensional** embeddings for both images and text. These embeddings are L2-normalized to unit vectors, which means:

- Cosine similarity can be computed as a simple dot product
- Distance can be computed as `1 - dot_product(a, b)`
- All embeddings have a magnitude of 1.0

## API Reference

### `MultimodalEngine`

- `load(model_dir)` - Load model from directory
- `embed_image(path)` - Embed single image
- `embed_batch_images(paths)` - Embed multiple images
- `embed_text(text)` - Embed single text
- `embed_batch_text(texts)` - Embed multiple texts
- `embed_image_binary(path)` - Binary quantized image embedding
- `embed_image_int8(path)` - Int8 quantized image embedding
- `embed_text_binary(text)` - Binary quantized text embedding
- `embed_text_int8(text)` - Int8 quantized text embedding
- `dimensions()` - Get embedding dimension (512)

### Utility Functions

- `cosine_similarity(a, b)` - Compute cosine similarity between two embeddings
- `cosine_distance(a, b)` - Compute cosine distance (1 - similarity)

## Error Handling

All methods return `Result<T, CxpError>`. Common errors include:

- `CxpError::Embedding` - ONNX runtime errors, model loading failures
- `CxpError::Io` - File not found, cannot read image

## Thread Safety

The `MultimodalEngine` requires mutable access (`&mut self`) for inference operations due to ONNX Runtime session requirements. For concurrent usage, wrap it in a mutex:

```rust
use std::sync::Mutex;
use cxp_core::multimodal::MultimodalEngine;

let engine = Mutex::new(MultimodalEngine::load("models/siglip2")?);

// In thread
let embedding = engine.lock().unwrap().embed_text("hello")?;
```

## License

See main project LICENSE file.

## References

- [SigLIP Paper](https://arxiv.org/abs/2303.15343) - Sigmoid Loss for Language Image Pre-training
- [ort Rust Crate](https://ort.pyke.io/) - ONNX Runtime Rust bindings
- Migration Guide: [ort v2.0 Migration](https://ort.pyke.io/migrating/v2)
