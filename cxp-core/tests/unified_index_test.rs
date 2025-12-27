//! Integration tests for unified cross-modal index
//!
//! These tests verify the unified index functionality with 512-dimensional embeddings

#[cfg(all(feature = "search", feature = "multimodal"))]
mod unified_index_tests {
    use cxp_core::{UnifiedIndex, HnswConfig};

    fn create_test_embedding_512(seed: f32) -> Vec<f32> {
        (0..512).map(|i| seed + (i as f32) * 0.001).collect()
    }

    #[test]
    fn test_unified_index_creation() {
        let config = HnswConfig::multimodal_float32();
        let index = UnifiedIndex::new(config);
        assert!(index.is_ok());

        let index = index.unwrap();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_unified_index_add_entries() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let text_emb = create_test_embedding_512(1.0);
        let image_emb = create_test_embedding_512(2.0);

        // Add text
        let result = index.add_text(1, &text_emb, 42, "document.txt");
        assert!(result.is_ok());

        // Add image
        let result = index.add_image(2, &image_emb, "photo.jpg");
        assert!(result.is_ok());

        assert_eq!(index.len(), 2);
        assert_eq!(index.text_count(), 1);
        assert_eq!(index.image_count(), 1);
    }

    #[test]
    fn test_unified_index_search() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let text_emb = create_test_embedding_512(1.0);
        let image_emb = create_test_embedding_512(2.0);

        index.add_text(1, &text_emb, 1, "doc.txt").unwrap();
        index.add_image(2, &image_emb, "photo.jpg").unwrap();

        // Search should find both
        let query = create_test_embedding_512(1.5);
        let results = index.search(&query, 2);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_unified_index_metadata() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let emb = create_test_embedding_512(1.0);
        index.add_text(1, &emb, 42, "test.txt").unwrap();

        let meta = index.get_metadata(1);
        assert!(meta.is_some());

        let meta = meta.unwrap();
        assert!(meta.is_text());
        assert_eq!(meta.file_path(), "test.txt");
    }
}
