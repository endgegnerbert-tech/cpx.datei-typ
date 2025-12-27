//! CXP CLI - Build and query CXP files
//!
//! Usage:
//!   cxp build <source-dir> <output.cxp> [--embeddings | --images] [--model <path>]
//!   cxp info <file.cxp>
//!   cxp list <file.cxp>
//!   cxp extract <file.cxp> <file-path> [output]
//!   cxp query <file.cxp> <search-term> [--top-k N]
//!   cxp search <file.cxp> [<query> | --image <path>] [--top-k N] [--result-type text|image|all] --model <path>
//!   cxp embed-image <image-path> --model <path> [--show-dims N]  (requires multimodal feature)
//!   cxp migrate <sqlite.db> <output.cxp> [--files <source-dir>]

mod migrate;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cxp_core::{CxpBuilder, CxpReader};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "cxp")]
#[command(about = "CXP - Universal AI Context Format", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a CXP file from a directory
    Build {
        /// Source directory to scan
        source: PathBuf,

        /// Output CXP file path
        output: PathBuf,

        /// Generate embeddings for semantic search
        #[arg(long)]
        embeddings: bool,

        /// Include images in the build (requires multimodal feature)
        #[arg(long)]
        images: bool,

        /// Path to embedding model directory (ONNX)
        /// For text: model.onnx + tokenizer.json
        /// For multimodal: image_encoder.onnx + text_encoder.onnx + tokenizer.json
        #[arg(long)]
        model: Option<PathBuf>,
    },

    /// Show information about a CXP file
    Info {
        /// CXP file to inspect
        file: PathBuf,
    },

    /// List files in a CXP archive
    List {
        /// CXP file to list
        file: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        long: bool,
    },

    /// Extract a file from a CXP archive
    Extract {
        /// CXP file
        file: PathBuf,

        /// Path of file to extract (within the CXP)
        path: String,

        /// Output path (default: stdout)
        output: Option<PathBuf>,
    },

    /// Query files in a CXP archive (keyword search)
    Query {
        /// CXP file to query
        file: PathBuf,

        /// Search term (keyword search)
        query: String,

        /// Number of results to return
        #[arg(short = 'k', long, default_value = "10")]
        top_k: usize,

        /// Case insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,
    },

    /// Semantic search in a CXP archive (requires embeddings)
    #[cfg(all(feature = "embeddings", feature = "search"))]
    Search {
        /// CXP file to search
        file: PathBuf,

        /// Search query (natural language, ignored if --image is used)
        #[arg(required_unless_present = "image")]
        query: Option<String>,

        /// Number of results
        #[arg(short = 'k', long, default_value = "10")]
        top_k: usize,

        /// Path to embedding model directory (ONNX)
        #[arg(long)]
        model: Option<PathBuf>,

        /// Filter results by type (text, image, or all)
        #[arg(long, default_value = "all")]
        result_type: String,

        /// Use an image as the search query (requires multimodal feature)
        #[arg(long)]
        image: Option<PathBuf>,
    },

    /// Migrate a SQLite database to CXP format
    Migrate {
        /// SQLite database file to migrate
        sqlite: PathBuf,

        /// Output CXP file path
        output: PathBuf,

        /// Optional source files directory to include
        #[arg(long)]
        files: Option<PathBuf>,
    },

    /// Generate and display embedding for an image (debugging)
    #[cfg(all(feature = "multimodal", feature = "search"))]
    EmbedImage {
        /// Path to the image file
        image: PathBuf,

        /// Path to SigLIP 2 model directory
        #[arg(long)]
        model: PathBuf,

        /// Show first N dimensions (default: 10)
        #[arg(long, default_value = "10")]
        show_dims: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Build { source, output, embeddings, images, model } => {
            build_cxp(&source, &output, embeddings, images, model.as_deref())
        }
        Commands::Info { file } => show_info(&file),
        Commands::List { file, long } => list_files(&file, long),
        Commands::Extract { file, path, output } => extract_file(&file, &path, output.as_deref()),
        Commands::Query { file, query, top_k, ignore_case } => {
            query_files(&file, &query, top_k, ignore_case)
        }
        #[cfg(all(feature = "embeddings", feature = "search"))]
        Commands::Search { file, query, top_k, model, result_type, image } => {
            search_semantic(&file, query.as_deref(), top_k, model.as_deref(), &result_type, image.as_deref())
        }
        Commands::Migrate { sqlite, output, files } => {
            migrate::migrate_sqlite_to_cxp(&sqlite, &output, files.as_deref())
        }
        #[cfg(all(feature = "multimodal", feature = "search"))]
        Commands::EmbedImage { image, model, show_dims } => {
            embed_image_command(&image, &model, show_dims)
        }
    }
}

fn build_cxp(
    source: &PathBuf,
    output: &PathBuf,
    embeddings: bool,
    #[allow(unused_variables)]
    images: bool,
    #[allow(unused_variables)]
    model: Option<&std::path::Path>,
) -> Result<()> {
    println!("Building CXP file...");
    println!("  Source: {}", source.display());
    println!("  Output: {}", output.display());

    // Check for incompatible feature combinations
    if images && embeddings {
        return Err(anyhow::anyhow!(
            "Cannot use both --embeddings and --images. Use --images with a multimodal model instead."
        ));
    }

    #[cfg(all(feature = "embeddings", feature = "search"))]
    if embeddings {
        println!("  Embeddings: enabled (text only)");
        if let Some(model_path) = model {
            println!("  Model: {}", model_path.display());
        }
    }

    #[cfg(feature = "multimodal")]
    if images {
        println!("  Images: enabled (multimodal)");
        if let Some(model_path) = model {
            println!("  Model: {}", model_path.display());
        }
    }

    println!();

    let start = Instant::now();

    let mut builder = CxpBuilder::new(source);

    // Enable images if requested
    #[cfg(feature = "multimodal")]
    if images {
        builder.with_images();
    }

    #[cfg(not(feature = "multimodal"))]
    if images {
        return Err(anyhow::anyhow!(
            "Image processing is not enabled. Rebuild cxp-cli with --features multimodal,search"
        ));
    }

    builder
        .scan()
        .context("Failed to scan directory")?
        .process()
        .context("Failed to process files")?;

    // Generate embeddings if requested
    #[cfg(all(feature = "embeddings", feature = "search"))]
    if embeddings {
        use cxp_core::EmbeddingModel;

        let model_path = model.ok_or_else(|| {
            anyhow::anyhow!(
                "Model path is required for embeddings. Use --model <path> to specify the model directory."
            )
        })?;

        builder
            .with_embeddings(model_path, EmbeddingModel::MiniLM)
            .context("Failed to initialize embeddings")?;
    }

    #[cfg(not(all(feature = "embeddings", feature = "search")))]
    if embeddings {
        return Err(anyhow::anyhow!(
            "Embeddings feature is not enabled. Rebuild cxp-cli with --features embeddings,search"
        ));
    }

    // Generate multimodal embeddings if images are enabled
    #[cfg(all(feature = "multimodal", feature = "search"))]
    if images {
        let model_path = model.ok_or_else(|| {
            anyhow::anyhow!(
                "Model path is required for multimodal embeddings. Use --model <path> to specify the SigLIP 2 model directory."
            )
        })?;

        builder
            .with_multimodal_embeddings(model_path)
            .context("Failed to initialize multimodal embeddings")?;
    }

    builder
        .build(output)
        .context("Failed to build CXP file")?;

    let duration = start.elapsed();

    println!();
    println!("Done in {:.2}s", duration.as_secs_f64());
    println!();

    // Show summary
    show_info(output)?;

    Ok(())
}

fn show_info(file: &PathBuf) -> Result<()> {
    let reader = CxpReader::open(file).context("Failed to open CXP file")?;
    let manifest = reader.manifest();

    println!("CXP File Information");
    println!("====================");
    println!();
    println!("Version:        {}", manifest.version);
    println!("Created:        {}", manifest.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!();
    println!("Statistics:");
    println!("  Total files:  {}", manifest.stats.total_files);
    println!("  Unique chunks:{}", manifest.stats.unique_chunks);
    println!(
        "  Original size:{:.2} MB",
        manifest.stats.original_size_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        "  CXP size:     {:.2} MB",
        manifest.stats.cxp_size_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        "  Compression:  {:.1}%",
        manifest.stats.compression_ratio * 100.0
    );
    println!(
        "  Dedup savings:{:.1}%",
        manifest.stats.dedup_savings_percent
    );
    println!();

    if !manifest.file_types.is_empty() {
        println!("File Types:");
        let mut types: Vec<_> = manifest.file_types.iter().collect();
        types.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        for (ext, info) in types.iter().take(10) {
            println!(
                "  .{:<10} {:>5} files ({:.2} KB)",
                ext,
                info.count,
                info.total_bytes as f64 / 1024.0
            );
        }
    }

    if !manifest.extensions.is_empty() {
        println!();
        println!("Extensions: {}", manifest.extensions.join(", "));
    }

    // Show embedding info if present
    if let Some(ref model) = manifest.embedding_model {
        println!();
        println!("Embeddings:");
        println!("  Model:      {}", model);
        if let Some(dim) = manifest.embedding_dim {
            println!("  Dimensions: {}", dim);
        }
    }

    Ok(())
}

fn list_files(file: &PathBuf, long: bool) -> Result<()> {
    let reader = CxpReader::open(file).context("Failed to open CXP file")?;

    let mut paths: Vec<_> = reader.file_paths();
    paths.sort();

    if long {
        println!("{:<60} {:>10} {:>6}", "PATH", "SIZE", "CHUNKS");
        println!("{}", "-".repeat(80));

        for path in paths {
            if let Some(entry) = reader.file_map.files.get(path) {
                println!(
                    "{:<60} {:>10} {:>6}",
                    path,
                    format_size(entry.size),
                    entry.chunks.len()
                );
            }
        }
    } else {
        for path in paths {
            println!("{}", path);
        }
    }

    Ok(())
}

fn extract_file(file: &PathBuf, path: &str, output: Option<&std::path::Path>) -> Result<()> {
    let reader = CxpReader::open(file).context("Failed to open CXP file")?;

    let content = reader.read_file(path).context("Failed to read file from CXP")?;

    match output {
        Some(output_path) => {
            std::fs::write(output_path, &content)?;
            println!("Extracted {} bytes to {}", content.len(), output_path.display());
        }
        None => {
            // Write to stdout
            std::io::stdout().write_all(&content)?;
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

/// Search result with file path, match count, and snippet
#[derive(Debug)]
struct SearchMatch {
    path: String,
    matches: usize,
    snippet: String,
    line_numbers: Vec<usize>,
}

fn query_files(file: &PathBuf, query: &str, top_k: usize, ignore_case: bool) -> Result<()> {
    let reader = CxpReader::open(file).context("Failed to open CXP file")?;

    println!("Searching for: \"{}\"", query);
    println!();

    let search_term = if ignore_case {
        query.to_lowercase()
    } else {
        query.to_string()
    };

    let mut results: Vec<SearchMatch> = Vec::new();

    // Search through all files
    for path in reader.file_paths() {
        if let Ok(content) = reader.read_file(path) {
            // Convert to string (skip binary content)
            if let Ok(text) = String::from_utf8(content) {
                // Count matches and collect line numbers
                let mut match_count = 0;
                let mut line_numbers = Vec::new();
                let mut snippet_lines = Vec::new();

                for (line_num, line) in text.lines().enumerate() {
                    let search_line = if ignore_case {
                        line.to_lowercase()
                    } else {
                        line.to_string()
                    };

                    if search_line.contains(&search_term) {
                        match_count += search_line.matches(&search_term).count();
                        line_numbers.push(line_num + 1);

                        // Collect first few matching lines for snippet
                        if snippet_lines.len() < 3 {
                            snippet_lines.push((line_num + 1, line.trim().to_string()));
                        }
                    }
                }

                if match_count > 0 {
                    // Create snippet from first 3 matching lines
                    let snippet = snippet_lines
                        .iter()
                        .map(|(num, line)| {
                            // Truncate long lines
                            let truncated = if line.len() > 80 {
                                format!("{}...", &line[..77])
                            } else {
                                line.clone()
                            };
                            format!("    {}:  {}", num, truncated)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    results.push(SearchMatch {
                        path: path.to_string(),
                        matches: match_count,
                        snippet,
                        line_numbers,
                    });
                }
            }
        }
    }

    // Sort by number of matches (descending)
    results.sort_by(|a, b| b.matches.cmp(&a.matches));

    // Show top-k results
    let display_count = results.len().min(top_k);

    if results.is_empty() {
        println!("No matches found.");
        return Ok(());
    }

    println!("Found {} file(s) with matches (showing top {}):", results.len(), display_count);
    println!();

    for (i, result) in results.iter().take(display_count).enumerate() {
        println!("{}. {} ({} match{})",
            i + 1,
            result.path,
            result.matches,
            if result.matches == 1 { "" } else { "es" }
        );

        if !result.snippet.is_empty() {
            println!("{}", result.snippet);
        }

        if result.line_numbers.len() > 3 {
            println!("    ... and {} more lines", result.line_numbers.len() - 3);
        }

        println!();
    }

    Ok(())
}

/// Perform semantic search using embeddings
#[cfg(all(feature = "embeddings", feature = "search"))]
fn search_semantic(
    file: &PathBuf,
    query: Option<&str>,
    top_k: usize,
    model: Option<&std::path::Path>,
    #[allow(unused_variables)]
    result_type: &str,
    #[allow(unused_variables)]
    image_query: Option<&std::path::Path>,
) -> Result<()> {
    use cxp_core::{EmbeddingEngine, EmbeddingModel};

    // Determine query type
    let is_image_query = image_query.is_some();

    if is_image_query {
        #[cfg(not(feature = "multimodal"))]
        {
            return Err(anyhow::anyhow!(
                "Image search requires multimodal feature. Rebuild with --features multimodal,search"
            ));
        }

        #[cfg(feature = "multimodal")]
        {
            println!("Image-to-multimodal search: {}", image_query.unwrap().display());
        }
    } else {
        println!("Semantic search: \"{}\"", query.unwrap_or(""));
    }

    println!();

    // Open CXP file
    let mut reader = CxpReader::open(file).context("Failed to open CXP file")?;

    // Check if file has embeddings
    if !reader.has_embeddings() {
        return Err(anyhow::anyhow!(
            "This CXP file has no embeddings. Use 'cxp build --embeddings --model <path>' to create one."
        ));
    }

    println!("Loading embeddings...");
    reader.load_embeddings().context("Failed to load embeddings")?;

    // Load embedding model and generate query embedding
    let model_path = model.ok_or_else(|| {
        anyhow::anyhow!(
            "Model path is required for search. Use --model <path> to specify the model directory."
        )
    })?;

    let query_embedding = if is_image_query {
        #[cfg(feature = "multimodal")]
        {
            use cxp_core::MultimodalEngine;

            println!("Loading multimodal model...");
            let mut engine = MultimodalEngine::load(model_path)
                .context("Failed to load multimodal model")?;

            println!("Encoding image...");
            engine.embed_image(image_query.unwrap())
                .context("Failed to encode image")?
        }
        #[cfg(not(feature = "multimodal"))]
        {
            unreachable!() // Already checked above
        }
    } else {
        println!("Loading embedding model...");
        let engine = EmbeddingEngine::load(model_path, EmbeddingModel::MiniLM)
            .context("Failed to load embedding model")?;

        println!("Encoding query...");
        engine.embed(query.unwrap()).context("Failed to encode query")?
    };

    // Search
    println!("Searching...");
    let results = reader
        .search_semantic(&query_embedding, top_k)
        .context("Search failed")?;

    if results.is_empty() {
        println!();
        println!("No results found.");
        return Ok(());
    }

    println!();
    println!("Found {} results:", results.len());
    println!();

    // Display results
    for (i, result) in results.iter().enumerate() {
        println!("{}. Chunk ID: {} (similarity: {:.4})", i + 1, result.id, -result.distance);

        // Try to get chunk content
        match reader.get_chunk_text(result.id) {
            Ok(text) => {
                // Show first few lines as preview
                let lines: Vec<&str> = text.lines().take(5).collect();
                for line in lines {
                    let truncated = if line.len() > 100 {
                        format!("{}...", &line[..97])
                    } else {
                        line.to_string()
                    };
                    println!("    {}", truncated);
                }
                if text.lines().count() > 5 {
                    println!("    ... ({} more lines)", text.lines().count() - 5);
                }
            }
            Err(_) => {
                println!("    [Could not retrieve chunk content]");
            }
        }

        println!();
    }

    Ok(())
}

/// Generate and display embedding for an image (debugging tool)
#[cfg(all(feature = "multimodal", feature = "search"))]
fn embed_image_command(
    image_path: &PathBuf,
    model_path: &PathBuf,
    show_dims: usize,
) -> Result<()> {
    use cxp_core::MultimodalEngine;

    println!("Loading SigLIP 2 model...");
    let mut engine = MultimodalEngine::load(model_path)
        .context("Failed to load multimodal model")?;

    println!("Embedding image: {}", image_path.display());
    let embedding = engine.embed_image(image_path)
        .context("Failed to embed image")?;

    println!();
    println!("Image Embedding");
    println!("===============");
    println!("Dimensions: {}", embedding.len());
    println!();

    // Display statistics
    let sum: f32 = embedding.iter().sum();
    let mean = sum / embedding.len() as f32;
    let variance: f32 = embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;
    let std_dev = variance.sqrt();
    let min = embedding.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = embedding.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    println!("Statistics:");
    println!("  Mean:     {:.6}", mean);
    println!("  Std Dev:  {:.6}", std_dev);
    println!("  Min:      {:.6}", min);
    println!("  Max:      {:.6}", max);
    println!();

    // Display first N dimensions
    let display_count = show_dims.min(embedding.len());
    println!("First {} dimensions:", display_count);
    for (i, value) in embedding.iter().take(display_count).enumerate() {
        println!("  [{}] = {:.6}", i, value);
    }

    if embedding.len() > display_count {
        println!("  ... ({} more dimensions)", embedding.len() - display_count);
    }

    println!();

    Ok(())
}
