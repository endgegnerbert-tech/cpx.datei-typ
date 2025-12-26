# CXP HNSW Vector Search

High-performance vector search using USearch's HNSW (Hierarchical Navigable Small World) implementation.

## Features

- Fast approximate nearest neighbor search
- Support for binary embeddings with Hamming distance
- Support for float32 embeddings with cosine similarity, L2, and inner product
- Persistent index with save/load operations
- Full integration with CXP embedding types
- Configurable index parameters for quality/speed tradeoff

## Usage

### Enable the Feature

Add to your `Cargo.toml`:

```toml
[dependencies]
cxp-core = { version = "1.0", features = ["search", "embeddings"] }
```

### Basic Float32 Search

```rust
use cxp_core::{HnswIndex, HnswConfig, DistanceMetric};

// Create an index for 384-dimensional embeddings (e.g., MiniLM)
let config = HnswConfig::float32_cosine(384);
let mut index = HnswIndex::new(config)?;

// Add vectors
let embedding1 = vec![0.1, 0.2, 0.3, ...]; // 384 dims
let embedding2 = vec![0.2, 0.3, 0.4, ...];

index.add_f32(1, &embedding1)?;
index.add_f32(2, &embedding2)?;

// Search for k nearest neighbors
let query = vec![0.15, 0.25, 0.35, ...];
let results = index.search_f32(&query, k: 10)?;

for result in results {
    println!("ID: {}, Distance: {}", result.id, result.distance);
}
```

### Binary Embedding Search

Binary embeddings are 32x smaller and much faster to search:

```rust
use cxp_core::{BinaryEmbedding, HnswConfig, HnswIndex};

// Create binary index (256 bits = 32 bytes)
let config = HnswConfig::binary(256);
let mut index = HnswIndex::new(config)?;

// Convert float embeddings to binary
let float_emb = vec![0.5, -0.3, 0.1, ...]; // 256 dims
let binary_emb = BinaryEmbedding::from_float(&float_emb);

// Add to index
index.add_binary_embedding(1, &binary_emb)?;

// Search
let query_binary = BinaryEmbedding::from_float(&query_float);
let results = index.search_binary_embedding(&query_binary, 10)?;
```

### Save and Load Index

```rust
// Save index to disk
index.save("my_index.usearch")?;

// Load index later
let config = HnswConfig::float32_cosine(384);
let loaded_index = HnswIndex::load("my_index.usearch", config)?;
```

### Batch Operations

```rust
// Add multiple vectors at once
let ids: Vec<u64> = vec![1, 2, 3, 4, 5];
let vectors: Vec<Vec<f32>> = vec![
    vec![...], // 384 dims
    vec![...],
    vec![...],
    vec![...],
    vec![...],
];

index.add_batch_f32(&ids, &vectors)?;
```

## Configuration

### HNSW Parameters

```rust
use cxp_core::{HnswConfig, DistanceMetric};

let config = HnswConfig {
    dimensions: 384,
    metric: DistanceMetric::Cosine,
    connectivity: 16,        // M: connections per layer (higher = better recall, more memory)
    expansion_add: 128,      // ef_construction: candidate list size during build
    expansion_search: 64,    // ef_search: candidate list size during search
};
```

### Distance Metrics

- **Cosine**: Best for normalized embeddings (most common)
- **L2**: Euclidean distance
- **IP**: Inner product (for pre-normalized vectors)
- **Hamming**: For binary embeddings only

### Performance Tuning

**For Better Recall (more accurate results):**
- Increase `connectivity` (e.g., 32)
- Increase `expansion_add` (e.g., 256)
- Increase `expansion_search` (e.g., 128)

**For Faster Search:**
- Decrease `expansion_search` (e.g., 32)
- Use binary embeddings with Hamming distance

**For Lower Memory Usage:**
- Decrease `connectivity` (e.g., 8)
- Use binary embeddings (32x smaller)
- Use Int8 embeddings (4x smaller)

## Two-Stage Search Pattern

For best quality/speed tradeoff, use binary embeddings for initial search, then rescore with Int8 or float32:

```rust
use cxp_core::{BinaryEmbedding, Int8Embedding};

// 1. Fast initial search with binary embeddings
let binary_index = HnswIndex::new(HnswConfig::binary(384))?;
let candidates = binary_index.search_binary_embedding(&query_binary, 100)?;

// 2. Rescore top candidates with Int8 for better accuracy
let int8_embeddings: Vec<Int8Embedding> = // stored separately
let mut rescored: Vec<_> = candidates.iter().map(|c| {
    let score = query_int8.dot_product(&int8_embeddings[c.id as usize]);
    (c.id, score)
}).collect();

rescored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
let top_10 = &rescored[..10];
```

## Integration with CXP Embeddings

```rust
use cxp_core::{EmbeddingEngine, EmbeddingModel, HnswIndex, HnswConfig};

// Generate embeddings
let engine = EmbeddingEngine::load("models/minilm", EmbeddingModel::MiniLM)?;
let texts = vec!["Document 1", "Document 2", "Document 3"];
let embeddings = engine.embed_batch(&texts)?;

// Create index
let config = HnswConfig::float32_cosine(engine.dimensions());
let mut index = HnswIndex::new(config)?;

// Add embeddings
for (id, embedding) in embeddings.iter().enumerate() {
    index.add_f32(id as u64, embedding)?;
}

// Search with new query
let query_embedding = engine.embed("search query")?;
let results = index.search_f32(&query_embedding, 5)?;
```

## Memory Requirements

For 1 million 384-dimensional float32 vectors with default settings (M=16):

- **Float32 Index**: ~2.5 GB
- **Binary Index**: ~80 MB (32x smaller)
- **Int8 Embeddings**: ~400 MB (for rescoring)

## Performance

On a typical workload (1M vectors, 384 dims):

| Configuration | Search Time | Recall@10 |
|--------------|-------------|-----------|
| Binary (ef=32) | 0.1 ms | 0.85 |
| Binary (ef=64) | 0.2 ms | 0.92 |
| Float32 (ef=64) | 0.5 ms | 0.95 |
| Float32 (ef=128) | 1.0 ms | 0.98 |

## Advanced Operations

### Remove Vectors

```rust
// Remove a specific vector
index.remove(42)?;

// Check if vector exists
if index.contains(42) {
    println!("Vector exists");
}
```

### Clear Index

```rust
// Remove all vectors
index.clear();
assert!(index.is_empty());
```

### Dynamic Search Parameters

```rust
// Adjust search quality at runtime
index.set_expansion_search(128); // Higher = better recall
let results = index.search_f32(&query, 10)?;
```

### Get Index Statistics

```rust
println!("Index size: {} vectors", index.len());
println!("Dimensions: {}", index.config().dimensions);
println!("Metric: {:?}", index.config().metric);
```

## Error Handling

```rust
use cxp_core::CxpError;

match index.add_f32(1, &embedding) {
    Ok(_) => println!("Added successfully"),
    Err(CxpError::Search(msg)) => eprintln!("Search error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Best Practices

1. **Normalize vectors** before adding to cosine similarity index
2. **Use binary embeddings** for initial search on large datasets
3. **Batch operations** for better performance when adding many vectors
4. **Save indices** to disk to avoid rebuilding
5. **Tune parameters** based on your recall/speed requirements
6. **Monitor memory** usage with large indices

## References

- [USearch Documentation](https://github.com/unum-cloud/usearch)
- [HNSW Paper](https://arxiv.org/abs/1603.09320)
- [Binary Quantization](https://arxiv.org/abs/2106.00882)
