//! Integration tests for HNSW index with embeddings

#[cfg(all(feature = "search", feature = "embeddings"))]
mod tests {
    use cxp_core::{
        BinaryEmbedding, HnswConfig, HnswIndex, DistanceMetric, Int8Embedding,
    };
    use tempfile::tempdir;

    #[test]
    fn test_binary_embedding_search() {
        // Create a binary index for 256-bit embeddings
        let config = HnswConfig::binary(256);
        let mut index = HnswIndex::new(config).expect("Failed to create index");

        // Create some test embeddings
        let embeddings = vec![
            vec![1.0f32; 256],  // All positive
            vec![-1.0f32; 256], // All negative
            vec![0.5f32; 256],  // All positive but smaller
        ];

        // Convert to binary embeddings
        let binary_embeddings: Vec<BinaryEmbedding> = embeddings
            .iter()
            .map(|e| BinaryEmbedding::from_float(e))
            .collect();

        // Add to index
        for (i, emb) in binary_embeddings.iter().enumerate() {
            index
                .add_binary_embedding(i as u64, emb)
                .expect("Failed to add embedding");
        }

        assert_eq!(index.len(), 3);

        // Search for nearest neighbor of first embedding
        let results = index
            .search_binary_embedding(&binary_embeddings[0], 2)
            .expect("Search failed");

        // First result should be exact match
        assert_eq!(results[0].id, 0);
        assert_eq!(results[0].distance, 0.0);

        // Second result should be embedding 2 (similar sign pattern)
        assert_eq!(results[1].id, 2);
    }

    #[test]
    fn test_float32_embedding_search() {
        // Create a float32 cosine index
        let config = HnswConfig::float32_cosine(128);
        let mut index = HnswIndex::new(config).expect("Failed to create index");

        // Create orthogonal vectors
        let mut vec1 = vec![0.0f32; 128];
        vec1[0] = 1.0;

        let mut vec2 = vec![0.0f32; 128];
        vec2[1] = 1.0;

        let mut vec3 = vec![0.0f32; 128];
        vec3[0] = 0.9;
        vec3[1] = 0.1;

        // Add to index
        index.add_f32(1, &vec1).expect("Failed to add vec1");
        index.add_f32(2, &vec2).expect("Failed to add vec2");
        index.add_f32(3, &vec3).expect("Failed to add vec3");

        assert_eq!(index.len(), 3);

        // Search for nearest to vec1
        let results = index.search_f32(&vec1, 2).expect("Search failed");

        // Should find vec1 as exact match
        assert_eq!(results[0].id, 1);
        assert!(results[0].distance < 0.001);

        // Second result should be vec3 (has component in vec1 direction)
        assert_eq!(results[1].id, 3);
    }

    #[test]
    fn test_save_and_load_index() {
        let dir = tempdir().expect("Failed to create temp dir");
        let index_path = dir.path().join("test.index");

        // Create and populate index
        {
            let config = HnswConfig::float32_cosine(64);
            let mut index = HnswIndex::new(config.clone()).expect("Failed to create index");

            let vectors = vec![
                vec![1.0f32; 64],
                vec![0.5f32; 64],
                vec![-0.5f32; 64],
            ];

            for (i, vec) in vectors.iter().enumerate() {
                index.add_f32(i as u64, vec).expect("Failed to add vector");
            }

            // Save index
            index.save(&index_path).expect("Failed to save index");
        }

        // Load index and verify
        {
            let config = HnswConfig::float32_cosine(64);
            let index = HnswIndex::load(&index_path, config).expect("Failed to load index");

            assert_eq!(index.len(), 3);

            // Search to verify data is intact
            let query = vec![1.0f32; 64];
            let results = index.search_f32(&query, 1).expect("Search failed");

            assert_eq!(results[0].id, 0);
        }
    }

    #[test]
    fn test_batch_operations() {
        let config = HnswConfig::float32_l2(32);
        let mut index = HnswIndex::new(config).expect("Failed to create index");

        // Prepare batch data
        let ids: Vec<u64> = (0..10).collect();
        let vectors: Vec<Vec<f32>> = (0..10)
            .map(|i| {
                let mut v = vec![0.0f32; 32];
                v[0] = i as f32;
                v
            })
            .collect();

        // Add batch
        index
            .add_batch_f32(&ids, &vectors)
            .expect("Batch add failed");

        assert_eq!(index.len(), 10);

        // Search
        let mut query = vec![0.0f32; 32];
        query[0] = 5.0;

        let results = index.search_f32(&query, 3).expect("Search failed");

        // Should find ID 5 as closest
        assert_eq!(results[0].id, 5);
    }

    #[test]
    fn test_remove_and_clear() {
        let config = HnswConfig::float32_cosine(16);
        let mut index = HnswIndex::new(config).expect("Failed to create index");

        // Add vectors
        for i in 0..5 {
            let vec = vec![i as f32; 16];
            index.add_f32(i, &vec).expect("Failed to add");
        }

        assert_eq!(index.len(), 5);
        assert!(index.contains(2));

        // Remove one
        index.remove(2).expect("Failed to remove");
        assert_eq!(index.len(), 4);
        assert!(!index.contains(2));

        // Clear all
        index.clear();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_different_distance_metrics() {
        // Test Hamming
        let config_hamming = HnswConfig {
            dimensions: 32, // 32 bytes = 256 bits
            metric: DistanceMetric::Hamming,
            ..Default::default()
        };
        let index_hamming = HnswIndex::new(config_hamming);
        assert!(index_hamming.is_ok());

        // Test Cosine
        let config_cosine = HnswConfig::float32_cosine(128);
        let index_cosine = HnswIndex::new(config_cosine);
        assert!(index_cosine.is_ok());

        // Test L2
        let config_l2 = HnswConfig::float32_l2(128);
        let index_l2 = HnswIndex::new(config_l2);
        assert!(index_l2.is_ok());

        // Test IP (Inner Product)
        let config_ip = HnswConfig {
            dimensions: 128,
            metric: DistanceMetric::IP,
            ..Default::default()
        };
        let index_ip = HnswIndex::new(config_ip);
        assert!(index_ip.is_ok());
    }

    #[test]
    fn test_search_result_similarity() {
        use cxp_core::SearchResult;

        let result = SearchResult {
            id: 1,
            distance: 0.2,
        };

        // Cosine: similarity = 1 - distance
        let cosine_sim = result.similarity(DistanceMetric::Cosine);
        assert!((cosine_sim - 0.8).abs() < 0.001);

        // IP: similarity = distance (already a similarity)
        let ip_sim = result.similarity(DistanceMetric::IP);
        assert!((ip_sim - 0.2).abs() < 0.001);

        // L2: similarity = 1 / (1 + distance)
        let l2_sim = result.similarity(DistanceMetric::L2);
        assert!((l2_sim - 1.0 / 1.2).abs() < 0.001);
    }

    #[test]
    fn test_dimension_validation() {
        let config = HnswConfig::float32_cosine(128);
        let mut index = HnswIndex::new(config).expect("Failed to create index");

        // Try to add vector with wrong dimensions
        let wrong_vec = vec![1.0f32; 64]; // Should be 128
        let result = index.add_f32(1, &wrong_vec);
        assert!(result.is_err());

        // Try to search with wrong dimensions
        let result = index.search_f32(&wrong_vec, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_int8_embedding_workflow() {
        // This test demonstrates a typical workflow:
        // 1. Generate embeddings (simulated)
        // 2. Create binary embeddings for fast search
        // 3. Use index for nearest neighbor search

        let embeddings = vec![
            vec![0.5f32; 384],
            vec![-0.5f32; 384],
            vec![0.3f32; 384],
        ];

        // Convert to Int8 for rescoring
        let int8_embeddings: Vec<Int8Embedding> = embeddings
            .iter()
            .map(|e| Int8Embedding::from_float(e))
            .collect();

        // Verify quantization preserves approximate relationships
        let dot_01 = int8_embeddings[0].dot_product(&int8_embeddings[1]);
        let dot_02 = int8_embeddings[0].dot_product(&int8_embeddings[2]);

        // Embedding 2 should be more similar to 0 than embedding 1
        assert!(dot_02 > dot_01);
    }
}
