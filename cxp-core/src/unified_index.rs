//! Unified Cross-Modal Search Index
//!
//! Combines text and image embeddings in a single HNSW index for cross-modal retrieval.
//! Uses SigLIP 2's shared 512-dimensional embedding space to enable:
//! - Text-to-image search
//! - Image-to-text search
//! - Unified multimodal search
//!
//! Features:
//! - Single unified index for all modalities
//! - Metadata tracking for entry types
//! - Type-specific filtering
//! - Persistent storage support

#[cfg(all(feature = "search", feature = "multimodal"))]
use crate::{CxpError, Result};

#[cfg(all(feature = "search", feature = "multimodal"))]
use crate::index::{HnswIndex, HnswConfig, SearchResult};

#[cfg(all(feature = "search", feature = "multimodal"))]
use std::collections::HashMap;

#[cfg(all(feature = "search", feature = "multimodal"))]
use std::path::Path;

#[cfg(all(feature = "search", feature = "multimodal"))]
use serde::{Deserialize, Serialize};

/// Type of entry in the unified index
#[cfg(all(feature = "search", feature = "multimodal"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    /// Text chunk entry
    Text {
        /// Unique chunk ID
        chunk_id: u64,
        /// Source file path
        file_path: String,
    },
    /// Image entry
    Image {
        /// Image file path
        file_path: String,
    },
}

impl EntryType {
    /// Check if this is a text entry
    pub fn is_text(&self) -> bool {
        matches!(self, EntryType::Text { .. })
    }

    /// Check if this is an image entry
    pub fn is_image(&self) -> bool {
        matches!(self, EntryType::Image { .. })
    }

    /// Get the file path for this entry
    pub fn file_path(&self) -> &str {
        match self {
            EntryType::Text { file_path, .. } => file_path,
            EntryType::Image { file_path } => file_path,
        }
    }
}

/// Search result with type information
#[cfg(all(feature = "search", feature = "multimodal"))]
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResultWithType {
    /// Vector ID
    pub id: u64,
    /// Distance to query (lower is more similar)
    pub distance: f32,
    /// Entry type and metadata
    pub entry_type: EntryType,
}

impl SearchResultWithType {
    /// Convert distance to similarity score (0.0 to 1.0)
    pub fn similarity(&self) -> f32 {
        // For cosine distance: similarity = 1 - distance
        1.0 - self.distance
    }
}

/// Unified index for cross-modal search
///
/// Stores both text and image embeddings in the same vector space,
/// enabling semantic search across modalities.
///
/// # Example
/// ```no_run
/// use cxp_core::unified_index::UnifiedIndex;
/// use cxp_core::index::HnswConfig;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a new unified index with 512-dim float32 embeddings
/// let config = HnswConfig::multimodal_float32();
/// let mut index = UnifiedIndex::new(config)?;
///
/// // Add text embedding
/// let text_embedding = vec![0.1; 512]; // Placeholder
/// index.add_text(1, &text_embedding, 42, "document.txt")?;
///
/// // Add image embedding
/// let image_embedding = vec![0.2; 512]; // Placeholder
/// index.add_image(2, &image_embedding, "photo.jpg")?;
///
/// // Search with text, find both images and text
/// let results = index.search(&text_embedding, 10)?;
///
/// // Search only images with text query
/// let image_results = index.search_images_only(&text_embedding, 5)?;
/// # Ok(())
/// # }
/// ```
#[cfg(all(feature = "search", feature = "multimodal"))]
pub struct UnifiedIndex {
    /// Underlying HNSW index
    hnsw: HnswIndex,
    /// Metadata for each entry
    metadata: HashMap<u64, EntryType>,
}

#[cfg(all(feature = "search", feature = "multimodal"))]
impl UnifiedIndex {
    /// Create a new unified index with the given configuration
    ///
    /// Recommended configs:
    /// - `HnswConfig::multimodal_float32()` for best accuracy (512-dim float32)
    /// - `HnswConfig::multimodal_binary()` for best compression (64 bytes)
    pub fn new(config: HnswConfig) -> Result<Self> {
        let hnsw = HnswIndex::new(config)?;
        let metadata = HashMap::new();

        Ok(Self { hnsw, metadata })
    }

    /// Add a text embedding to the index
    ///
    /// # Arguments
    /// * `id` - Unique ID for this vector (must be unique across all entries)
    /// * `embedding` - 512-dimensional embedding vector
    /// * `chunk_id` - Text chunk identifier
    /// * `file_path` - Source file path
    pub fn add_text(
        &mut self,
        id: u64,
        embedding: &[f32],
        chunk_id: u64,
        file_path: &str,
    ) -> Result<()> {
        self.hnsw.add_f32(id, embedding)?;
        self.metadata.insert(
            id,
            EntryType::Text {
                chunk_id,
                file_path: file_path.to_string(),
            },
        );
        Ok(())
    }

    /// Add a text embedding using binary quantization
    ///
    /// More memory-efficient than float32, with minimal accuracy loss.
    pub fn add_text_binary(
        &mut self,
        id: u64,
        bits: &[u8],
        chunk_id: u64,
        file_path: &str,
    ) -> Result<()> {
        self.hnsw.add_binary(id, bits)?;
        self.metadata.insert(
            id,
            EntryType::Text {
                chunk_id,
                file_path: file_path.to_string(),
            },
        );
        Ok(())
    }

    /// Add an image embedding to the index
    ///
    /// # Arguments
    /// * `id` - Unique ID for this vector (must be unique across all entries)
    /// * `embedding` - 512-dimensional embedding vector
    /// * `file_path` - Image file path
    pub fn add_image(&mut self, id: u64, embedding: &[f32], file_path: &str) -> Result<()> {
        self.hnsw.add_f32(id, embedding)?;
        self.metadata.insert(
            id,
            EntryType::Image {
                file_path: file_path.to_string(),
            },
        );
        Ok(())
    }

    /// Add an image embedding using binary quantization
    ///
    /// More memory-efficient than float32, with minimal accuracy loss.
    pub fn add_image_binary(&mut self, id: u64, bits: &[u8], file_path: &str) -> Result<()> {
        self.hnsw.add_binary(id, bits)?;
        self.metadata.insert(
            id,
            EntryType::Image {
                file_path: file_path.to_string(),
            },
        );
        Ok(())
    }

    /// Cross-modal search: find both text and images
    ///
    /// Returns all matching entries regardless of type, sorted by similarity.
    ///
    /// # Arguments
    /// * `query` - Query embedding (can be from text or image)
    /// * `top_k` - Number of results to return
    pub fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<SearchResultWithType>> {
        let results = self.hnsw.search_f32(query, top_k)?;
        self.attach_metadata(results)
    }

    /// Cross-modal search with binary query
    pub fn search_binary(&self, bits: &[u8], top_k: usize) -> Result<Vec<SearchResultWithType>> {
        let results = self.hnsw.search_binary(bits, top_k)?;
        self.attach_metadata(results)
    }

    /// Search only for images
    ///
    /// Filters results to return only image entries. Useful for text-to-image search.
    pub fn search_images_only(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResultWithType>> {
        // Search more than requested to account for filtering
        let search_k = top_k * 3; // Heuristic: search 3x to get enough images
        let all_results = self.hnsw.search_f32(query, search_k)?;
        let results_with_metadata = self.attach_metadata(all_results)?;

        // Filter to images only
        let filtered: Vec<_> = results_with_metadata
            .into_iter()
            .filter(|r| r.entry_type.is_image())
            .take(top_k)
            .collect();

        Ok(filtered)
    }

    /// Search only for text
    ///
    /// Filters results to return only text entries. Useful for image-to-text search.
    pub fn search_text_only(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResultWithType>> {
        // Search more than requested to account for filtering
        let search_k = top_k * 3; // Heuristic: search 3x to get enough text
        let all_results = self.hnsw.search_f32(query, search_k)?;
        let results_with_metadata = self.attach_metadata(all_results)?;

        // Filter to text only
        let filtered: Vec<_> = results_with_metadata
            .into_iter()
            .filter(|r| r.entry_type.is_text())
            .take(top_k)
            .collect();

        Ok(filtered)
    }

    /// Remove an entry from the index
    pub fn remove(&mut self, id: u64) -> Result<()> {
        self.hnsw.remove(id)?;
        self.metadata.remove(&id);
        Ok(())
    }

    /// Check if an entry exists in the index
    pub fn contains(&self, id: u64) -> bool {
        self.hnsw.contains(id)
    }

    /// Get metadata for an entry
    pub fn get_metadata(&self, id: u64) -> Option<&EntryType> {
        self.metadata.get(&id)
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.hnsw.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.hnsw.is_empty()
    }

    /// Get number of text entries
    pub fn text_count(&self) -> usize {
        self.metadata
            .values()
            .filter(|e| e.is_text())
            .count()
    }

    /// Get number of image entries
    pub fn image_count(&self) -> usize {
        self.metadata
            .values()
            .filter(|e| e.is_image())
            .count()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.hnsw.clear();
        self.metadata.clear();
    }

    /// Save index and metadata to disk
    ///
    /// Saves two files:
    /// - `{path}.index` - HNSW index data
    /// - `{path}.meta` - Metadata (JSON)
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let base_path = path.as_ref();

        // Save HNSW index
        let index_path = base_path.with_extension("index");
        self.hnsw.save(&index_path)?;

        // Save metadata as JSON
        let meta_path = base_path.with_extension("meta");
        let meta_json = serde_json::to_string(&self.metadata)
            .map_err(|e| CxpError::Search(format!("Failed to serialize metadata: {}", e)))?;
        std::fs::write(&meta_path, meta_json)
            .map_err(|e| CxpError::Search(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// Load index and metadata from disk
    pub fn load<P: AsRef<Path>>(path: P, config: HnswConfig) -> Result<Self> {
        let base_path = path.as_ref();

        // Load HNSW index
        let index_path = base_path.with_extension("index");
        let hnsw = HnswIndex::load(&index_path, config)?;

        // Load metadata
        let meta_path = base_path.with_extension("meta");
        let meta_json = std::fs::read_to_string(&meta_path)
            .map_err(|e| CxpError::Search(format!("Failed to read metadata: {}", e)))?;
        let metadata: HashMap<u64, EntryType> = serde_json::from_str(&meta_json)
            .map_err(|e| CxpError::Search(format!("Failed to deserialize metadata: {}", e)))?;

        Ok(Self { hnsw, metadata })
    }

    /// Set the search expansion parameter (ef_search)
    ///
    /// Higher values = better recall, slower search
    pub fn set_expansion_search(&mut self, expansion: usize) {
        self.hnsw.set_expansion_search(expansion);
    }

    /// Helper to attach metadata to search results
    fn attach_metadata(
        &self,
        results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResultWithType>> {
        results
            .into_iter()
            .map(|r| {
                let entry_type = self
                    .metadata
                    .get(&r.id)
                    .ok_or_else(|| {
                        CxpError::Search(format!("Missing metadata for ID {}", r.id))
                    })?
                    .clone();

                Ok(SearchResultWithType {
                    id: r.id,
                    distance: r.distance,
                    entry_type,
                })
            })
            .collect()
    }
}

#[cfg(all(test, feature = "search", feature = "multimodal"))]
mod tests {
    use super::*;

    fn create_test_embedding(seed: f32) -> Vec<f32> {
        // Create a simple 512-dim embedding for testing
        (0..512).map(|i| seed + (i as f32) * 0.001).collect()
    }

    #[test]
    fn test_create_unified_index() {
        let config = HnswConfig::multimodal_float32();
        let index = UnifiedIndex::new(config).unwrap();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
        assert_eq!(index.text_count(), 0);
        assert_eq!(index.image_count(), 0);
    }

    #[test]
    fn test_add_text_and_image() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let text_emb = create_test_embedding(1.0);
        let image_emb = create_test_embedding(2.0);

        index.add_text(1, &text_emb, 42, "doc.txt").unwrap();
        index.add_image(2, &image_emb, "photo.jpg").unwrap();

        assert_eq!(index.len(), 2);
        assert_eq!(index.text_count(), 1);
        assert_eq!(index.image_count(), 1);
    }

    #[test]
    fn test_cross_modal_search() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        // Add several entries
        let text_emb1 = create_test_embedding(1.0);
        let text_emb2 = create_test_embedding(1.1);
        let image_emb1 = create_test_embedding(2.0);
        let image_emb2 = create_test_embedding(2.1);

        index.add_text(1, &text_emb1, 1, "doc1.txt").unwrap();
        index.add_text(2, &text_emb2, 2, "doc2.txt").unwrap();
        index.add_image(3, &image_emb1, "photo1.jpg").unwrap();
        index.add_image(4, &image_emb2, "photo2.jpg").unwrap();

        // Search should return mixed results
        let query = create_test_embedding(1.05);
        let results = index.search(&query, 4).unwrap();

        assert_eq!(results.len(), 4);
        // Results should contain both text and images
        let has_text = results.iter().any(|r| r.entry_type.is_text());
        let has_image = results.iter().any(|r| r.entry_type.is_image());
        assert!(has_text);
        assert!(has_image);
    }

    #[test]
    fn test_search_images_only() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let text_emb = create_test_embedding(1.0);
        let image_emb1 = create_test_embedding(1.1);
        let image_emb2 = create_test_embedding(1.2);

        index.add_text(1, &text_emb, 1, "doc.txt").unwrap();
        index.add_image(2, &image_emb1, "photo1.jpg").unwrap();
        index.add_image(3, &image_emb2, "photo2.jpg").unwrap();

        // Search with text query, get only images
        let results = index.search_images_only(&text_emb, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.entry_type.is_image()));
    }

    #[test]
    fn test_search_text_only() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let text_emb1 = create_test_embedding(1.0);
        let text_emb2 = create_test_embedding(1.1);
        let image_emb = create_test_embedding(1.2);

        index.add_text(1, &text_emb1, 1, "doc1.txt").unwrap();
        index.add_text(2, &text_emb2, 2, "doc2.txt").unwrap();
        index.add_image(3, &image_emb, "photo.jpg").unwrap();

        // Search with image query, get only text
        let results = index.search_text_only(&image_emb, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.entry_type.is_text()));
    }

    #[test]
    fn test_metadata_access() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let emb = create_test_embedding(1.0);
        index.add_text(1, &emb, 42, "test.txt").unwrap();

        let meta = index.get_metadata(1).unwrap();
        assert!(meta.is_text());
        assert_eq!(meta.file_path(), "test.txt");

        if let EntryType::Text { chunk_id, .. } = meta {
            assert_eq!(*chunk_id, 42);
        } else {
            panic!("Expected Text entry");
        }
    }

    #[test]
    fn test_remove_entry() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let emb = create_test_embedding(1.0);
        index.add_text(1, &emb, 1, "doc.txt").unwrap();

        assert!(index.contains(1));
        assert_eq!(index.len(), 1);

        index.remove(1).unwrap();

        assert!(!index.contains(1));
        assert_eq!(index.len(), 0);
        assert!(index.get_metadata(1).is_none());
    }

    #[test]
    fn test_clear_index() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let emb1 = create_test_embedding(1.0);
        let emb2 = create_test_embedding(2.0);

        index.add_text(1, &emb1, 1, "doc.txt").unwrap();
        index.add_image(2, &emb2, "photo.jpg").unwrap();

        assert_eq!(index.len(), 2);

        index.clear();

        assert_eq!(index.len(), 0);
        assert_eq!(index.text_count(), 0);
        assert_eq!(index.image_count(), 0);
    }

    #[test]
    fn test_similarity_score() {
        let config = HnswConfig::multimodal_float32();
        let mut index = UnifiedIndex::new(config).unwrap();

        let emb = create_test_embedding(1.0);
        index.add_text(1, &emb, 1, "doc.txt").unwrap();

        let results = index.search(&emb, 1).unwrap();
        assert_eq!(results.len(), 1);

        // Searching for the same embedding should give high similarity
        let similarity = results[0].similarity();
        assert!(similarity > 0.99, "Similarity was {}", similarity);
    }

    #[test]
    fn test_entry_type_helpers() {
        let text = EntryType::Text {
            chunk_id: 1,
            file_path: "doc.txt".to_string(),
        };
        assert!(text.is_text());
        assert!(!text.is_image());
        assert_eq!(text.file_path(), "doc.txt");

        let image = EntryType::Image {
            file_path: "photo.jpg".to_string(),
        };
        assert!(image.is_image());
        assert!(!image.is_text());
        assert_eq!(image.file_path(), "photo.jpg");
    }

    #[test]
    fn test_binary_quantization() {
        let config = HnswConfig::multimodal_binary();
        let mut index = UnifiedIndex::new(config).unwrap();

        // Create 64-byte binary vector (512 bits)
        let bits = vec![0b10101010u8; 64];

        index.add_text_binary(1, &bits, 1, "doc.txt").unwrap();
        index.add_image_binary(2, &bits, "photo.jpg").unwrap();

        assert_eq!(index.len(), 2);

        // Search with binary query
        let results = index.search_binary(&bits, 2).unwrap();
        assert_eq!(results.len(), 2);
    }
}
