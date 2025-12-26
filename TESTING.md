# CXP Testing Guide

This document describes the test suite for the CXP (Universal AI Context Format) project.

## Test Overview

The CXP project includes comprehensive tests covering:

1. **Unit Tests** - Tests for individual modules (chunker, compress, dedup, manifest, etc.)
2. **Integration Tests** - End-to-end tests for the complete CXP workflow
3. **Clippy Lints** - Static analysis for code quality

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Only Unit Tests

```bash
cargo test --lib
```

### Run Only Integration Tests

```bash
cargo test --test integration_test
```

### Run Specific Test

```bash
cargo test test_name
```

### Run With Output

```bash
cargo test -- --nocapture
```

### Run Clippy

```bash
cargo clippy -- -D warnings
```

### Build Project

```bash
cargo build --release
```

## Test Coverage

### Unit Tests by Module

#### `chunker.rs`
- ✓ `test_chunk_content` - Verifies content chunking with FastCDC
- ✓ `test_compute_hash` - Tests SHA-256 hash computation
- ✓ `test_empty_content` - Handles empty content gracefully

#### `compress.rs`
- ✓ `test_compress_decompress` - Verifies Zstandard compression roundtrip
- ✓ `test_compression_ratio` - Tests compression effectiveness
- ✓ `test_empty_data` - Handles empty data compression

#### `dedup.rs`
- ✓ `test_deduplication` - Verifies duplicate chunk detection
- ✓ `test_stats` - Tests deduplication statistics calculation

#### `manifest.rs`
- ✓ `test_manifest_creation` - Tests manifest initialization
- ✓ `test_manifest_serialization` - Verifies MessagePack serialization

#### `format.rs`
- ✓ `test_file_entry_serialization` - Tests file entry serialization

#### `embeddings.rs` (optional feature)
- ✓ `test_binary_quantization` - Tests binary embedding quantization
- ✓ `test_binary_hamming_distance` - Verifies Hamming distance calculation
- ✓ `test_int8_quantization` - Tests Int8 quantization
- ✓ `test_int8_dot_product` - Verifies dot product approximation
- ✓ `test_quantized_embeddings_size` - Tests size calculations

### Integration Tests

Located in `/cxp-core/tests/integration_test.rs`:

#### Builder Tests
- ✓ `test_cxp_builder_scan` - Tests directory scanning
- ✓ `test_cxp_builder_process` - Tests file processing and chunking
- ✓ `test_cxp_builder_build` - Tests CXP file creation

#### Reader Tests
- ✓ `test_cxp_reader_open` - Tests CXP file opening
- ✓ `test_cxp_reader_list_files` - Tests file listing
- ✓ `test_cxp_reader_read_file` - Tests file content extraction
- ✓ `test_cxp_reader_extract_and_verify` - Verifies content integrity

#### Chunking & Deduplication Tests
- ✓ `test_chunking_consistency` - Verifies chunking produces consistent results
- ✓ `test_deduplication` - Tests duplicate content detection and savings
- ✓ `test_large_file_chunking` - Tests chunking of large files (>100KB)

#### Compression Tests
- ✓ `test_compression_effectiveness` - Verifies compression reduces file size

#### Edge Case Tests
- ✓ `test_file_not_found_error` - Tests error handling for missing files
- ✓ `test_empty_directory` - Tests behavior with empty directories

#### Manifest Tests
- ✓ `test_manifest_file_types` - Verifies file type detection and categorization

#### End-to-End Tests
- ✓ `test_complete_workflow` - Tests the full scan→process→build→read workflow

## Test Data

Integration tests use temporary directories created with `tempfile` crate:
- **Small files**: 6 test files (Rust, Markdown, TOML, JSON)
- **Large files**: 100KB text files for chunking tests
- **Duplicate files**: Multiple files with identical content for dedup testing

## Expected Test Results

All tests should pass with the following characteristics:

### Performance Expectations
- Deduplication savings: >0% for duplicate content
- Compression ratio: <1.0 (compressed < original)
- Chunk count: Multiple chunks for files >4KB (AVG_CHUNK_SIZE)

### File Format Expectations
- CXP files are valid ZIP archives
- Contains `manifest.msgpack` and `file_map.msgpack`
- Contains `chunks/*.zst` compressed chunk files
- Version matches `cxp_core::VERSION`

## Continuous Integration

For CI/CD pipelines, run:

```bash
# Full test suite with warnings as errors
cargo test
cargo clippy -- -D warnings
cargo build --release
```

## Debugging Failed Tests

### View Test Output
```bash
cargo test -- --nocapture --test-threads=1
```

### Run Specific Test with Backtrace
```bash
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

### Check Test File Artifacts
Integration tests use `tempfile::TempDir` which automatically cleans up. To inspect artifacts, modify tests to use a fixed path temporarily.

## Adding New Tests

When adding new features:

1. **Unit Tests**: Add to the module's `#[cfg(test)]` section
2. **Integration Tests**: Add to `cxp-core/tests/integration_test.rs`
3. **Documentation**: Update this file with test descriptions

### Test Template

```rust
#[test]
fn test_new_feature() -> Result<()> {
    // Arrange
    let test_data = setup_test_data();

    // Act
    let result = perform_operation(test_data)?;

    // Assert
    assert_eq!(result.expected_field, expected_value);

    Ok(())
}
```

## Test Maintenance

- Run tests before committing: `cargo test`
- Update tests when changing public APIs
- Add regression tests for bug fixes
- Keep test data minimal but representative

## Known Limitations

1. **Embeddings Tests**: Require ONNX models (large files), tested separately with `#[cfg(feature = "embeddings")]`
2. **Platform-Specific**: Some tests may behave differently on Windows vs Unix (path separators, etc.)
3. **Timing**: Tests do not include performance benchmarks (use `cargo bench` for that)

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Integration Testing in Rust](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Clippy Documentation](https://github.com/rust-lang/rust-clippy)
