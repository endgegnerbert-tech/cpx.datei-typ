# CXP Integration Tests

This directory contains integration tests for the CXP (Universal AI Context Format) library.

## Test Files

### `integration_test.rs`
Comprehensive end-to-end tests covering the complete CXP workflow:

#### Test Categories

1. **Builder Tests**
   - Directory scanning
   - File processing and chunking
   - CXP file creation

2. **Reader Tests**
   - Opening CXP files
   - Listing files
   - Reading file content
   - Content integrity verification

3. **Feature Tests**
   - Chunking consistency
   - Deduplication effectiveness
   - Compression performance
   - Large file handling (>100KB)

4. **Edge Cases**
   - Error handling (file not found)
   - Empty directories
   - File type detection

5. **End-to-End**
   - Complete workflow from scan to extract

## Running Tests

### Run all integration tests
```bash
cargo test --test integration_test
```

### Run specific test
```bash
cargo test test_cxp_builder_scan
```

### Run with output
```bash
cargo test --test integration_test -- --nocapture
```

### Run with backtrace
```bash
RUST_BACKTRACE=1 cargo test --test integration_test
```

## Test Structure

Each test follows the Arrange-Act-Assert pattern:

```rust
#[test]
fn test_feature() -> Result<()> {
    // Arrange: Create test data
    let test_dir = create_test_directory()?;

    // Act: Perform operations
    let mut builder = CxpBuilder::new(test_dir.path());
    builder.scan()?.process()?.build(&output_path)?;

    // Assert: Verify results
    assert!(output_path.exists());

    Ok(())
}
```

## Test Helpers

### `create_test_directory() -> Result<TempDir>`
Creates a temporary directory with sample files:
- `src/main.rs` - Rust source file
- `src/lib.rs` - Rust library file
- `src/utils.rs` - Rust utility file
- `README.md` - Markdown documentation
- `config.toml` - TOML configuration
- `data.json` - JSON data file

The temporary directory is automatically cleaned up after tests complete.

## Coverage

Integration tests verify:
- ✅ File scanning and filtering
- ✅ Content-defined chunking (FastCDC)
- ✅ Deduplication (SHA-256 based)
- ✅ Compression (Zstandard)
- ✅ ZIP container creation
- ✅ Manifest generation
- ✅ File map creation
- ✅ Content extraction
- ✅ Error handling

## Adding New Tests

When adding new integration tests:

1. **Follow naming convention**: `test_<feature>_<aspect>`
2. **Use test helpers**: Reuse `create_test_directory()` when possible
3. **Clean up resources**: Use `TempDir` for temporary files
4. **Add documentation**: Update this README

Example:
```rust
#[test]
fn test_new_feature() -> Result<()> {
    let test_dir = create_test_directory()?;
    let output_dir = TempDir::new().map_err(|e| CxpError::Io(e))?;
    let output_path = output_dir.path().join("test.cxp");

    // Test implementation here

    Ok(())
}
```

## Dependencies

Integration tests use:
- `tempfile` - Temporary directory creation
- Test data is generated on-the-fly (no external files needed)

## Notes

- Tests use real filesystem operations
- Each test is independent and isolated
- Tests run in parallel by default (use `--test-threads=1` to serialize)
- Temporary files are cleaned up automatically
