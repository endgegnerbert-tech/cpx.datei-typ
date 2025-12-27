//! CXP File Format - ZIP container with structured content
//!
//! Structure:
//! ```text
//! file.cxp (ZIP)
//! ├── manifest.msgpack     # Metadata & stats
//! ├── file_map.msgpack     # File -> Chunk references
//! ├── chunks/
//! │   ├── 0001.zst         # Compressed chunks
//! │   ├── 0002.zst
//! │   └── ...
//! ├── embeddings/          # Optional: Semantic search support
//! │   ├── binary.bin       # Binary quantized embeddings
//! │   ├── int8.bin         # Int8 quantized embeddings for rescoring
//! │   └── index.hnsw       # HNSW index for fast search
//! └── extensions/          # Optional app-specific data
//!     └── ...
//! ```

use crate::chunker::{chunk_content, Chunk, ChunkRef};
use crate::compress::{compress, decompress};
use crate::dedup::ChunkStore;
use crate::manifest::Manifest;
use crate::extensions::{Extension, ExtensionManager};
use crate::{is_text_file, CxpError, Result};

// Embedding types (shared across embeddings and search features)
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
use crate::{BinaryEmbedding, Int8Embedding, QuantizedEmbeddings};

// Search-specific types
#[cfg(all(feature = "embeddings", feature = "search"))]
use crate::{EmbeddingEngine, EmbeddingModel, HnswConfig, HnswIndex};

// Serialization functions for embeddings
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
use crate::{serialize_binary_embeddings, deserialize_binary_embeddings, serialize_int8_embeddings, deserialize_int8_embeddings};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter, CompressionMethod};

/// File map - maps file paths to their chunk references
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMap {
    /// Map of file path -> list of chunk references
    pub files: HashMap<String, FileEntry>,
}

/// Entry for a single file in the file map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Original file path (relative)
    pub path: String,
    /// File extension
    pub extension: String,
    /// Original file size
    pub size: u64,
    /// Chunk references that make up this file
    pub chunks: Vec<ChunkRef>,
}

/// A CXP file handle
pub struct CxpFile {
    /// The manifest
    pub manifest: Manifest,
    /// File map
    pub file_map: FileMap,
    /// Chunk store
    pub chunk_store: ChunkStore,
}

/// Builder for creating CXP files
pub struct CxpBuilder {
    /// Source directory to scan
    source_dir: PathBuf,
    /// Files to include
    files: Vec<PathBuf>,
    /// Manifest
    manifest: Manifest,
    /// File map
    file_map: FileMap,
    /// Chunk store with deduplication
    chunk_store: ChunkStore,
    /// Extension manager for app-specific data
    extension_manager: ExtensionManager,
    /// Embedding engine (optional)
    #[cfg(all(feature = "embeddings", feature = "search"))]
    embedding_engine: Option<EmbeddingEngine>,
    /// Chunk embeddings (optional)
    #[cfg(all(feature = "embeddings", feature = "search"))]
    chunk_embeddings: Option<QuantizedEmbeddings>,
    /// HNSW search index (optional)
    #[cfg(all(feature = "embeddings", feature = "search"))]
    search_index: Option<HnswIndex>,
}

impl CxpBuilder {
    /// Create a new CXP builder for a directory
    pub fn new<P: AsRef<Path>>(source_dir: P) -> Self {
        Self {
            source_dir: source_dir.as_ref().to_path_buf(),
            files: Vec::new(),
            manifest: Manifest::new(),
            file_map: FileMap::default(),
            chunk_store: ChunkStore::new(),
            extension_manager: ExtensionManager::new(),
            #[cfg(all(feature = "embeddings", feature = "search"))]
            embedding_engine: None,
            #[cfg(all(feature = "embeddings", feature = "search"))]
            chunk_embeddings: None,
            #[cfg(all(feature = "embeddings", feature = "search"))]
            search_index: None,
        }
    }

    /// Scan the source directory for files
    pub fn scan(&mut self) -> Result<&mut Self> {
        tracing::info!("Scanning directory: {:?}", self.source_dir);

        self.files = WalkDir::new(&self.source_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                // Filter by text extensions
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(is_text_file)
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        tracing::info!("Found {} files to process", self.files.len());
        Ok(self)
    }

    /// Process all scanned files
    pub fn process(&mut self) -> Result<&mut Self> {
        let source_dir = self.source_dir.clone();

        // Process files and collect chunks
        let results: Vec<_> = self.files
            .iter()
            .filter_map(|path| {
                self.process_file(path, &source_dir).ok()
            })
            .collect();

        // Add to chunk store and file map
        for (entry, chunks) in results {
            let chunk_refs = self.chunk_store.add_many(chunks);

            // Update manifest with file type info
            self.manifest.add_file_type(&entry.extension, &entry.path, entry.size);

            // Store file entry with chunk refs
            let entry_with_refs = FileEntry {
                chunks: chunk_refs,
                ..entry
            };
            self.file_map.files.insert(entry_with_refs.path.clone(), entry_with_refs);
        }

        // Update manifest stats
        let dedup_stats = self.chunk_store.stats();
        self.manifest.stats.total_files = self.file_map.files.len();
        self.manifest.stats.unique_chunks = dedup_stats.unique_chunks;
        self.manifest.stats.original_size_bytes = dedup_stats.total_bytes as u64;
        self.manifest.stats.dedup_savings_percent = dedup_stats.savings_percent();

        tracing::info!(
            "Processed {} files, {} unique chunks, {:.1}% dedup savings",
            self.manifest.stats.total_files,
            self.manifest.stats.unique_chunks,
            self.manifest.stats.dedup_savings_percent
        );

        Ok(self)
    }

    /// Enable embedding generation (requires both "embeddings" and "search" features)
    ///
    /// This loads an embedding model and will generate embeddings for all chunks
    /// during the build process. The embeddings are stored in the CXP file along
    /// with a HNSW index for fast semantic search.
    ///
    /// # Arguments
    /// * `model_path` - Path to the directory containing model.onnx and tokenizer.json
    /// * `model` - The embedding model to use (e.g., EmbeddingModel::MiniLM)
    ///
    /// # Example
    /// ```ignore
    /// builder
    ///     .scan()?
    ///     .with_embeddings("./models/all-MiniLM-L6-v2", EmbeddingModel::MiniLM)?
    ///     .process()?
    ///     .build("output.cxp")?;
    /// ```
    #[cfg(all(feature = "embeddings", feature = "search"))]
    pub fn with_embeddings<P: AsRef<Path>>(
        &mut self,
        model_path: P,
        model: EmbeddingModel,
    ) -> Result<&mut Self> {
        tracing::info!("Loading embedding model: {}", model.name());

        let engine = EmbeddingEngine::load(model_path, model)?;

        // Update manifest with embedding info
        self.manifest.embedding_model = Some(model.name().to_string());
        self.manifest.embedding_dim = Some(model.dimensions());

        self.embedding_engine = Some(engine);

        Ok(self)
    }

    /// Generate embeddings for all chunks
    ///
    /// This is automatically called during `build()` if embeddings are enabled.
    /// You can also call it manually after `process()` to inspect the embeddings.
    #[cfg(all(feature = "embeddings", feature = "search"))]
    pub fn generate_embeddings(&mut self) -> Result<&mut Self> {
        let engine = self.embedding_engine.as_ref()
            .ok_or_else(|| CxpError::Embedding(
                "Embedding engine not initialized. Call with_embeddings() first.".to_string()
            ))?;

        tracing::info!("Generating embeddings for {} unique chunks", self.chunk_store.len());

        // Collect all chunk texts
        let chunks: Vec<_> = self.chunk_store.chunks().collect();
        let chunk_texts: Vec<&str> = chunks
            .iter()
            .map(|c| {
                std::str::from_utf8(&c.data)
                    .unwrap_or("[binary data]")
            })
            .collect();

        // Process in batches to avoid OOM
        const BATCH_SIZE: usize = 32;
        let mut all_embeddings = Vec::new();

        for batch in chunk_texts.chunks(BATCH_SIZE) {
            let embeddings = engine.embed_batch(batch)?;
            all_embeddings.extend(embeddings);
        }

        tracing::info!("Generated {} embeddings", all_embeddings.len());

        // Create quantized embeddings
        let quantized = QuantizedEmbeddings::from_floats(&all_embeddings);

        tracing::info!(
            "Quantized embeddings size: {:.2} MB (binary) + {:.2} MB (int8)",
            quantized.binary.iter().map(|e| e.size_bytes()).sum::<usize>() as f64 / 1024.0 / 1024.0,
            quantized.int8.iter().map(|e| e.size_bytes()).sum::<usize>() as f64 / 1024.0 / 1024.0
        );

        // Build HNSW index for binary embeddings
        let config = HnswConfig::binary(engine.dimensions());
        let mut index = HnswIndex::new(config)?;

        tracing::info!("Building HNSW index...");

        for (i, binary_emb) in quantized.binary.iter().enumerate() {
            index.add_binary_embedding(i as u64, binary_emb)?;
        }

        tracing::info!("HNSW index built with {} vectors", index.len());

        self.chunk_embeddings = Some(quantized);
        self.search_index = Some(index);

        Ok(self)
    }

    /// Add an extension with data to this CXP file
    ///
    /// The extension will be registered and its data will be stored in the
    /// extensions/{namespace}/ directory in the CXP file.
    ///
    /// # Arguments
    /// * `ext` - An object implementing the Extension trait
    /// * `data` - A HashMap of file names to their data (e.g., "conversations.msgpack" -> bytes)
    ///
    /// # Example
    /// ```ignore
    /// use std::collections::HashMap;
    ///
    /// let mut builder = CxpBuilder::new("./src");
    /// let contextai = ContextAIExtension::new();
    /// let data = contextai.to_extension_data()?;
    ///
    /// builder.add_extension(&contextai, data)?;
    /// ```
    pub fn add_extension<E: Extension + Clone>(
        &mut self,
        ext: &E,
        data: HashMap<String, Vec<u8>>,
    ) -> Result<&mut Self> {
        // Register the extension
        self.extension_manager.register(ext.clone());

        // Add all data files
        for (key, bytes) in data {
            self.extension_manager.write_data(ext.namespace(), &key, &bytes)?;
        }

        // Update manifest to include this extension
        if !self.manifest.extensions.contains(&ext.namespace().to_string()) {
            self.manifest.extensions.push(ext.namespace().to_string());
        }

        tracing::info!(
            "Added extension '{}' (v{}) with {} data files",
            ext.namespace(),
            ext.version(),
            self.extension_manager.list_data_keys(ext.namespace()).len()
        );

        Ok(self)
    }

    /// Process a single file
    fn process_file(&self, path: &Path, base_dir: &Path) -> Result<(FileEntry, Vec<Chunk>)> {
        // Read file content
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let mut content = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut content)?;

        // Get relative path
        let relative_path = path
            .strip_prefix(base_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Get extension
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Chunk the content
        let chunks = chunk_content(&content);

        let entry = FileEntry {
            path: relative_path,
            extension,
            size: metadata.len(),
            chunks: Vec::new(), // Will be filled in with refs later
        };

        Ok((entry, chunks))
    }

    /// Build and write the CXP file
    pub fn build<P: AsRef<Path>>(&mut self, output_path: P) -> Result<()> {
        let output_path = output_path.as_ref();
        tracing::info!("Building CXP file: {:?}", output_path);

        // Generate embeddings if engine is set but embeddings haven't been generated yet
        #[cfg(all(feature = "embeddings", feature = "search"))]
        if self.embedding_engine.is_some() && self.chunk_embeddings.is_none() {
            self.generate_embeddings()?;
        }

        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::<()>::default()
            .compression_method(CompressionMethod::Stored); // We compress chunks ourselves

        // Write manifest
        let manifest_data = self.manifest.to_msgpack()?;
        zip.start_file("manifest.msgpack", options.clone())?;
        zip.write_all(&manifest_data)?;

        // Write file map
        let file_map_data = rmp_serde::to_vec(&self.file_map)?;
        zip.start_file("file_map.msgpack", options.clone())?;
        zip.write_all(&file_map_data)?;

        // Write chunks
        let chunks: Vec<_> = self.chunk_store.chunks().collect();
        let total_chunks = chunks.len();

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_name = format!("chunks/{}.zst", chunk.id());
            let compressed = compress(&chunk.data)?;

            zip.start_file(&chunk_name, options.clone())?;
            zip.write_all(&compressed)?;

            if (i + 1) % 100 == 0 || i + 1 == total_chunks {
                tracing::debug!("Written {}/{} chunks", i + 1, total_chunks);
            }
        }

        // Write embeddings if present
        #[cfg(all(feature = "embeddings", feature = "search"))]
        if let Some(ref embeddings) = self.chunk_embeddings {
            tracing::info!("Writing embeddings to CXP file...");

            // Write binary embeddings
            let binary_data = serialize_binary_embeddings(&embeddings.binary)?;
            zip.start_file("embeddings/binary.bin", options.clone())?;
            zip.write_all(&binary_data)?;

            // Write int8 embeddings
            let int8_data = serialize_int8_embeddings(&embeddings.int8)?;
            zip.start_file("embeddings/int8.bin", options.clone())?;
            zip.write_all(&int8_data)?;

            // Mark that we have embeddings
            if !self.manifest.extensions.contains(&"embeddings".to_string()) {
                self.manifest.extensions.push("embeddings".to_string());
            }

            tracing::info!("Embeddings written successfully");
        }

        // Write HNSW index if present
        #[cfg(all(feature = "embeddings", feature = "search"))]
        if let Some(ref index) = self.search_index {
            tracing::info!("Writing HNSW index to CXP file...");

            // Save index to a temporary file first (USearch limitation)
            let temp_dir = std::env::temp_dir();
            let temp_index_path = temp_dir.join(format!("cxp_index_{}.hnsw", uuid::Uuid::new_v4()));

            index.save(&temp_index_path)?;

            // Read the index file and write to ZIP
            let mut index_file = File::open(&temp_index_path)?;
            let mut index_data = Vec::new();
            index_file.read_to_end(&mut index_data)?;

            zip.start_file("embeddings/index.hnsw", options.clone())?;
            zip.write_all(&index_data)?;

            // Clean up temp file
            std::fs::remove_file(&temp_index_path)?;

            tracing::info!("HNSW index written successfully ({} vectors)", index.len());
        }

        // Write extension data if present
        if !self.extension_manager.list_extensions().is_empty() {
            tracing::info!("Writing extension data to CXP file...");

            // Write extension manifests
            for manifest in self.extension_manager.manifests().values() {
                let manifest_path = format!("extensions/{}/manifest.msgpack", manifest.namespace);
                let manifest_data = manifest.to_msgpack()?;
                zip.start_file(&manifest_path, options.clone())?;
                zip.write_all(&manifest_data)?;
            }

            // Write extension data files
            for (namespace, data_map) in self.extension_manager.all_data() {
                for (key, data) in data_map {
                    let data_path = format!("extensions/{}/{}", namespace, key);
                    zip.start_file(&data_path, options.clone())?;
                    zip.write_all(data)?;
                }
            }

            tracing::info!(
                "Written {} extensions to CXP file",
                self.extension_manager.list_extensions().len()
            );
        }

        zip.finish()?;

        // Update manifest with final size
        let final_size = std::fs::metadata(output_path)?.len();
        self.manifest.stats.cxp_size_bytes = final_size;
        self.manifest.stats.compression_ratio =
            final_size as f64 / self.manifest.stats.original_size_bytes as f64;

        tracing::info!(
            "CXP file created: {:.2} MB (compression ratio: {:.2}%)",
            final_size as f64 / 1024.0 / 1024.0,
            self.manifest.stats.compression_ratio * 100.0
        );

        Ok(())
    }
}

/// Reader for CXP files
pub struct CxpReader {
    /// The manifest
    pub manifest: Manifest,
    /// File map
    pub file_map: FileMap,
    /// ZIP archive handle
    archive_path: PathBuf,
    /// Extension manager for reading app-specific data
    extension_manager: ExtensionManager,
    /// Cached HNSW index for semantic search
    #[cfg(all(feature = "embeddings", feature = "search"))]
    search_index: Option<HnswIndex>,
    /// Cached embeddings for rescoring
    #[cfg(all(feature = "embeddings", feature = "search"))]
    embeddings: Option<QuantizedEmbeddings>,
}

impl CxpReader {
    /// Open a CXP file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mut archive = ZipArchive::new(file)?;

        // Read manifest
        let manifest = {
            let mut manifest_file = archive.by_name("manifest.msgpack")?;
            let mut data = Vec::new();
            manifest_file.read_to_end(&mut data)?;
            Manifest::from_msgpack(&data)?
        };

        // Read file map
        let file_map = {
            let mut file_map_file = archive.by_name("file_map.msgpack")?;
            let mut data = Vec::new();
            file_map_file.read_to_end(&mut data)?;
            rmp_serde::from_slice(&data)?
        };

        // Load extension data if present
        let mut extension_manager = ExtensionManager::new();

        // Iterate through all files in the ZIP archive to find extensions
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let file_name = file.name().to_string();

            // Check if this is an extension file
            if file_name.starts_with("extensions/") {
                let parts: Vec<&str> = file_name.split('/').collect();
                if parts.len() >= 3 {
                    let namespace = parts[1];
                    let file_key = parts[2..].join("/");

                    // Read the file data
                    let mut data = Vec::new();
                    drop(file); // Close the borrowed file
                    let mut file = archive.by_index(i)?;
                    file.read_to_end(&mut data)?;

                    if file_key == "manifest.msgpack" {
                        // Load extension manifest
                        if let Ok(ext_manifest) = crate::extensions::ExtensionManifest::from_msgpack(&data) {
                            extension_manager.load_manifest(ext_manifest);
                        }
                    } else {
                        // Load extension data
                        extension_manager.load_data(namespace.to_string(), file_key, data);
                    }
                }
            }
        }

        if !extension_manager.list_extensions().is_empty() {
            tracing::info!(
                "Loaded {} extensions from CXP file",
                extension_manager.list_extensions().len()
            );
        }

        Ok(Self {
            manifest,
            file_map,
            archive_path: path,
            extension_manager,
            #[cfg(all(feature = "embeddings", feature = "search"))]
            search_index: None,
            #[cfg(all(feature = "embeddings", feature = "search"))]
            embeddings: None,
        })
    }

    /// Get the manifest
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get all file paths
    pub fn file_paths(&self) -> Vec<&str> {
        self.file_map.files.keys().map(|s| s.as_str()).collect()
    }

    /// Read a file's content by reconstructing from chunks
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let entry = self.file_map.files.get(path)
            .ok_or_else(|| CxpError::FileNotFound(path.to_string()))?;

        let file = File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut content = Vec::with_capacity(entry.size as usize);

        for chunk_ref in &entry.chunks {
            let chunk_name = format!("chunks/{}.zst", &chunk_ref.hash[..16]);
            let mut chunk_file = archive.by_name(&chunk_name)?;

            let mut compressed = Vec::new();
            chunk_file.read_to_end(&mut compressed)?;

            let decompressed = decompress(&compressed)?;
            content.extend_from_slice(&decompressed);
        }

        Ok(content)
    }

    /// Check if this CXP file has embeddings
    #[cfg(any(feature = "embeddings", feature = "embeddings-wasm", feature = "search"))]
    pub fn has_embeddings(&self) -> bool {
        self.manifest.embedding_model.is_some()
            && self.manifest.extensions.contains(&"embeddings".to_string())
    }

    /// Check if embeddings are available (without feature flags - returns false)
    #[cfg(not(any(feature = "embeddings", feature = "embeddings-wasm", feature = "search")))]
    pub fn has_embeddings(&self) -> bool {
        false
    }

    /// Load embeddings as an EmbeddingStore without caching
    ///
    /// Returns the embeddings without loading the HNSW index or caching them.
    /// Use this if you only need to access the embeddings directly.
    #[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
    pub fn get_embedding_store(&self) -> Result<crate::EmbeddingStore> {
        if !self.has_embeddings() {
            return Err(CxpError::Embedding(
                "This CXP file does not contain embeddings".to_string()
            ));
        }

        tracing::info!("Loading embeddings from CXP file...");

        let file = File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Load binary embeddings
        let binary_embeddings = {
            let mut file = archive.by_name("embeddings/binary.bin")?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            deserialize_binary_embeddings(&data)?
        };

        // Load int8 embeddings
        let int8_embeddings = {
            let mut file = archive.by_name("embeddings/int8.bin")?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            deserialize_int8_embeddings(&data)?
        };

        let dimensions = self.manifest.embedding_dim
            .ok_or_else(|| CxpError::Embedding("No embedding dimension in manifest".to_string()))?;

        tracing::info!("Loaded {} embeddings", binary_embeddings.len());

        Ok(crate::EmbeddingStore::new(
            binary_embeddings,
            int8_embeddings,
            dimensions,
        ))
    }

    /// Load embeddings and search index into memory
    ///
    /// This must be called before using semantic search functions.
    /// The embeddings and index are cached for subsequent searches.
    #[cfg(all(feature = "embeddings", feature = "search"))]
    pub fn load_embeddings(&mut self) -> Result<()> {
        if !self.has_embeddings() {
            return Err(CxpError::Embedding(
                "This CXP file does not contain embeddings".to_string()
            ));
        }

        if self.search_index.is_some() {
            return Ok(());  // Already loaded
        }

        tracing::info!("Loading embeddings from CXP file...");

        let file = File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Load binary embeddings
        let binary_embeddings = {
            let mut file = archive.by_name("embeddings/binary.bin")?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            deserialize_binary_embeddings(&data)?
        };

        // Load int8 embeddings
        let int8_embeddings = {
            let mut file = archive.by_name("embeddings/int8.bin")?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            deserialize_int8_embeddings(&data)?
        };

        tracing::info!("Loaded {} embeddings", binary_embeddings.len());

        self.embeddings = Some(QuantizedEmbeddings {
            binary: binary_embeddings,
            int8: int8_embeddings,
        });

        // Load HNSW index
        let file = File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut index_file = archive.by_name("embeddings/index.hnsw")?;
        let mut index_data = Vec::new();
        index_file.read_to_end(&mut index_data)?;

        // Save to temp file (USearch limitation)
        let temp_dir = std::env::temp_dir();
        let temp_index_path = temp_dir.join(format!("cxp_index_{}.hnsw", uuid::Uuid::new_v4()));

        let mut temp_file = File::create(&temp_index_path)?;
        temp_file.write_all(&index_data)?;
        drop(temp_file);

        // Load index
        let dimensions = self.manifest.embedding_dim
            .ok_or_else(|| CxpError::Embedding("No embedding dimension in manifest".to_string()))?;

        let config = HnswConfig::binary(dimensions);
        let index = HnswIndex::load(&temp_index_path, config)?;

        // Clean up temp file
        std::fs::remove_file(&temp_index_path)?;

        tracing::info!("Loaded HNSW index with {} vectors", index.len());

        self.search_index = Some(index);

        Ok(())
    }

    /// Perform semantic search using a query embedding
    ///
    /// Returns the top-k most similar chunks by ID.
    /// You must call `load_embeddings()` first.
    ///
    /// # Arguments
    /// * `query_embedding` - The query vector (should match the model's dimensions)
    /// * `top_k` - Number of results to return
    ///
    /// # Returns
    /// Vector of (chunk_id, similarity_score) tuples, sorted by similarity (highest first)
    #[cfg(all(feature = "embeddings", feature = "search"))]
    pub fn search_semantic(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let index = self.search_index.as_ref()
            .ok_or_else(|| CxpError::Search(
                "Embeddings not loaded. Call load_embeddings() first.".to_string()
            ))?;

        let embeddings = self.embeddings.as_ref()
            .ok_or_else(|| CxpError::Search(
                "Embeddings not loaded. Call load_embeddings() first.".to_string()
            ))?;

        // Convert query to binary for fast initial search
        let query_binary = BinaryEmbedding::from_float(query_embedding);

        // Search with HNSW (binary)
        let candidates = index.search_binary_embedding(&query_binary, top_k * 2)?;

        // Rescore with Int8 for better accuracy
        let query_int8 = Int8Embedding::from_float(query_embedding);

        let mut rescored: Vec<_> = candidates
            .iter()
            .map(|result| {
                let chunk_id = result.id as usize;
                let score = if chunk_id < embeddings.int8.len() {
                    embeddings.int8[chunk_id].dot_product(&query_int8)
                } else {
                    0.0
                };
                SearchResult {
                    id: result.id,
                    distance: -score,  // Negate for sorting (higher is better)
                }
            })
            .collect();

        // Sort by score (descending)
        rescored.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        // Take top-k and fix the distance sign
        Ok(rescored
            .into_iter()
            .take(top_k)
            .map(|mut r| {
                r.distance = -r.distance;
                r
            })
            .collect())
    }

    /// Get chunk text by ID
    ///
    /// This is useful for retrieving the actual content of chunks found by semantic search.
    #[cfg(all(feature = "embeddings", feature = "search"))]
    pub fn get_chunk_text(&self, chunk_id: u64) -> Result<String> {
        let file = File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        // We need to find the chunk by iterating through the chunk store
        // For now, we'll use the chunk hash format
        let chunk_name = format!("chunks/{:016x}.zst", chunk_id);

        let mut chunk_file = archive.by_name(&chunk_name)
            .or_else(|_| {
                // Try alternative naming if the first format doesn't work
                // This is a fallback - in practice you'd maintain a chunk ID -> hash mapping
                Err(CxpError::FileNotFound(format!("Chunk {} not found", chunk_id)))
            })?;

        let mut compressed = Vec::new();
        chunk_file.read_to_end(&mut compressed)?;

        let decompressed = decompress(&compressed)?;

        String::from_utf8(decompressed)
            .map_err(|e| CxpError::Serialization(format!("Invalid UTF-8 in chunk: {}", e)))
    }

    /// List all extension namespaces in this CXP file
    ///
    /// Returns a vector of extension namespace strings (e.g., ["contextai", "custom"])
    pub fn list_extensions(&self) -> Vec<String> {
        self.extension_manager
            .list_extensions()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Read extension data from the CXP file
    ///
    /// # Arguments
    /// * `namespace` - The extension namespace (e.g., "contextai")
    /// * `key` - The data key within the namespace (e.g., "conversations.msgpack")
    ///
    /// # Returns
    /// The raw bytes of the extension data file
    ///
    /// # Example
    /// ```ignore
    /// let reader = CxpReader::open("example.cxp")?;
    /// let data = reader.read_extension("contextai", "conversations.msgpack")?;
    /// let conversations: Vec<Conversation> = rmp_serde::from_slice(&data)?;
    /// ```
    pub fn read_extension(&self, namespace: &str, key: &str) -> Result<Vec<u8>> {
        self.extension_manager.read_data(namespace, key)
    }

    /// Get extension manifest for a specific namespace
    ///
    /// Returns the extension's metadata including version and description
    pub fn get_extension_manifest(&self, namespace: &str) -> Option<&crate::extensions::ExtensionManifest> {
        self.extension_manager.get_manifest(namespace)
    }

    /// List all data keys for a specific extension namespace
    ///
    /// Returns a vector of data file names within the extension
    pub fn list_extension_keys(&self, namespace: &str) -> Vec<String> {
        self.extension_manager
            .list_data_keys(namespace)
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_serialization() {
        let entry = FileEntry {
            path: "src/main.rs".to_string(),
            extension: "rs".to_string(),
            size: 1000,
            chunks: vec![],
        };

        let data = rmp_serde::to_vec(&entry).unwrap();
        let restored: FileEntry = rmp_serde::from_slice(&data).unwrap();

        assert_eq!(restored.path, entry.path);
    }
}
