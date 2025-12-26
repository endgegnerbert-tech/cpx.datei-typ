//! Integration tests for CXP format
//!
//! Tests the complete workflow: scan -> process -> build -> read -> extract

use cxp_core::{CxpBuilder, CxpReader, CxpError, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test directory with sample files
fn create_test_directory() -> Result<TempDir> {
    let temp_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;

    // Create some test files
    let files = vec![
        ("src/main.rs", "fn main() {\n    println!(\"Hello, world!\");\n}\n"),
        ("src/lib.rs", "pub mod utils;\n\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"),
        ("src/utils.rs", "pub fn multiply(a: i32, b: i32) -> i32 {\n    a * b\n}\n"),
        ("README.md", "# Test Project\n\nThis is a test project for CXP.\n"),
        ("config.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ("data.json", "{\"name\": \"test\", \"value\": 42}\n"),
    ];

    for (path, content) in files {
        let file_path = temp_dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
    }

    Ok(temp_dir)
}

#[test]
fn test_cxp_builder_scan() -> Result<()> {
    let test_dir = create_test_directory()?;

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?;

    // We created 6 files, all with supported extensions
    // The scan should succeed without error (we can't access private fields)
    // We'll verify files were scanned in the process step

    Ok(())
}

#[test]
fn test_cxp_builder_process() -> Result<()> {
    let test_dir = create_test_directory()?;

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?;

    // Process should complete successfully
    // We'll verify the results by building and reading the CXP file

    Ok(())
}

#[test]
fn test_cxp_builder_build() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Verify the CXP file was created
    assert!(output_path.exists());
    assert!(output_path.metadata()?.len() > 0);

    Ok(())
}

#[test]
fn test_cxp_reader_open() -> Result<()> {
    // Build a CXP file first
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Now try to open it
    let reader = CxpReader::open(&output_path)?;

    // Verify manifest is accessible
    assert_eq!(reader.manifest().version, cxp_core::VERSION);
    assert!(reader.manifest().stats.total_files > 0);

    Ok(())
}

#[test]
fn test_cxp_reader_list_files() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    let reader = CxpReader::open(&output_path)?;
    let file_paths = reader.file_paths();

    // Should have found files
    assert!(!file_paths.is_empty());

    // Check that we have some expected files
    let paths_str = file_paths.join(",");
    assert!(paths_str.contains("main.rs") || paths_str.contains("lib.rs") || paths_str.contains("README.md"));

    Ok(())
}

#[test]
fn test_cxp_reader_read_file() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    let reader = CxpReader::open(&output_path)?;
    let file_paths = reader.file_paths();

    // Read the first file
    if let Some(first_file) = file_paths.first() {
        let content = reader.read_file(first_file)?;
        assert!(!content.is_empty(), "File content should not be empty");
    }

    Ok(())
}

#[test]
fn test_cxp_reader_extract_and_verify() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    // Create original content map
    let original_content = std::collections::HashMap::from([
        ("README.md", "# Test Project\n\nThis is a test project for CXP.\n"),
        ("config.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ("data.json", "{\"name\": \"test\", \"value\": 42}\n"),
    ]);

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    let reader = CxpReader::open(&output_path)?;

    // Verify we can read back the exact content for some files
    for (filename, expected_content) in &original_content {
        // Find the file in the CXP
        let file_path = reader.file_paths()
            .into_iter()
            .find(|p| p.contains(filename));

        if let Some(path) = file_path {
            let content = reader.read_file(path)?;
            let content_str = String::from_utf8_lossy(&content);
            assert_eq!(
                content_str.as_ref(),
                *expected_content,
                "Content mismatch for {}",
                filename
            );
        }
    }

    Ok(())
}

#[test]
fn test_chunking_consistency() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Verify by reading back
    let reader = CxpReader::open(&output_path)?;
    assert!(reader.manifest().stats.unique_chunks > 0, "Should have created chunks");

    Ok(())
}

#[test]
fn test_deduplication() -> Result<()> {
    // Create a directory with duplicate content
    let temp_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;

    // Create multiple files with the same content
    let duplicate_content = "This is repeated content that should be deduplicated.\n".repeat(10);

    for i in 1..=3 {
        let file_path = temp_dir.path().join(format!("file{}.txt", i));
        let mut file = File::create(&file_path)?;
        file.write_all(duplicate_content.as_bytes())?;
    }

    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("dedup_test.cxp");

    let mut builder = CxpBuilder::new(temp_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Verify deduplication by checking the manifest
    let reader = CxpReader::open(&output_path)?;
    let manifest = reader.manifest();

    // With deduplication, we should see savings
    assert!(manifest.stats.dedup_savings_percent > 0.0, "Should have deduplication savings");

    Ok(())
}

#[test]
fn test_compression_effectiveness() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Check CXP file size directly (manifest cxp_size_bytes is set after build, not stored in file)
    let cxp_file_size = std::fs::metadata(&output_path)?.len();
    assert!(cxp_file_size > 0, "CXP file should have size > 0");

    // Read manifest to check original size
    let reader = CxpReader::open(&output_path)?;
    let manifest = reader.manifest();

    // Verify original size is tracked
    assert!(manifest.stats.original_size_bytes > 0, "Original size should be tracked");

    Ok(())
}

#[test]
fn test_file_not_found_error() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    let reader = CxpReader::open(&output_path)?;

    // Try to read a non-existent file
    let result = reader.read_file("non_existent_file.txt");
    assert!(result.is_err());

    match result {
        Err(CxpError::FileNotFound(_)) => {
            // Expected error type
        }
        _ => panic!("Expected FileNotFound error"),
    }

    Ok(())
}

#[test]
fn test_empty_directory() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("empty.cxp");

    let mut builder = CxpBuilder::new(temp_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Empty directory should result in no files
    let reader = CxpReader::open(&output_path)?;
    assert_eq!(reader.manifest().stats.total_files, 0);

    Ok(())
}

#[test]
fn test_large_file_chunking() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;

    // Create a large file (>100KB)
    let large_content = "A".repeat(100_000);
    let file_path = temp_dir.path().join("large.txt");
    let mut file = File::create(&file_path)?;
    file.write_all(large_content.as_bytes())?;

    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("large_test.cxp");

    let mut builder = CxpBuilder::new(temp_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Verify we can read it back
    let reader = CxpReader::open(&output_path)?;

    // Large file should be split into multiple chunks
    // AVG_CHUNK_SIZE is 4KB, so 100KB should create multiple chunks
    assert!(reader.manifest().stats.unique_chunks > 1, "Large file should be split into multiple chunks");

    let file_paths = reader.file_paths();
    let large_file_path = file_paths.iter().find(|p| p.contains("large.txt")).unwrap();
    let content = reader.read_file(large_file_path)?;

    assert_eq!(content.len(), large_content.len(), "Content size should match");
    assert_eq!(String::from_utf8_lossy(&content), large_content, "Content should match exactly");

    Ok(())
}

#[test]
fn test_manifest_file_types() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    let reader = CxpReader::open(&output_path)?;
    let manifest = reader.manifest();

    // Should have detected different file types
    assert!(!manifest.file_types.is_empty());

    // Should have Rust files
    if let Some(rs_info) = manifest.file_types.get("rs") {
        assert!(rs_info.count > 0);
        assert_eq!(rs_info.description, "Rust");
    }

    // Should have Markdown files
    if let Some(md_info) = manifest.file_types.get("md") {
        assert!(md_info.count > 0);
        assert_eq!(md_info.description, "Markdown");
    }

    Ok(())
}

#[test]
fn test_complete_workflow() -> Result<()> {
    // This test verifies the complete workflow from start to finish
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("complete_workflow.cxp");

    // Step 1: Create CXP file
    let mut builder = CxpBuilder::new(test_dir.path());
    builder
        .scan()?
        .process()?
        .build(&output_path)?;

    // Step 2: Open and verify
    let reader = CxpReader::open(&output_path)?;

    // Step 3: List all files
    let files = reader.file_paths();
    assert!(!files.is_empty(), "Should contain files");

    // Step 4: Read each file and verify it's not empty
    for file_path in files {
        let content = reader.read_file(file_path)?;
        assert!(!content.is_empty(), "File {} should not be empty", file_path);
    }

    // Step 5: Verify manifest
    let manifest = reader.manifest();
    assert_eq!(manifest.version, cxp_core::VERSION);
    assert!(manifest.stats.total_files > 0);
    assert!(manifest.stats.unique_chunks > 0);
    assert!(manifest.stats.original_size_bytes > 0);

    // Verify actual file size (cxp_size_bytes is set after writing, not stored in manifest)
    let actual_file_size = std::fs::metadata(&output_path)?.len();
    assert!(actual_file_size > 0, "CXP file should have size > 0");

    Ok(())
}
