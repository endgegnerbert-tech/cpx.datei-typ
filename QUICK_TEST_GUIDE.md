# CXP Quick Test Guide

## Run All Tests
```bash
cargo test
```

## Run Specific Test Suites

### Unit Tests Only
```bash
cargo test --lib
```

### Integration Tests Only
```bash
cargo test --test integration_test
```

### Specific Test
```bash
cargo test test_name
```

## View Output

### Show Print Statements
```bash
cargo test -- --nocapture
```

### Show Backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Single-Threaded (Easier to Debug)
```bash
cargo test -- --test-threads=1
```

## Code Quality

### Run Clippy
```bash
cargo clippy -- -D warnings
```

### Build Release
```bash
cargo build --release
```

### Build and Run All Tests
```bash
cargo build && cargo test && cargo clippy
```

## Quick Test Runner Script
```bash
chmod +x test_runner.sh
./test_runner.sh
```

## Test by Module

### Test Chunking
```bash
cargo test chunker
```

### Test Compression
```bash
cargo test compress
```

### Test Deduplication
```bash
cargo test dedup
```

### Test Manifest
```bash
cargo test manifest
```

### Test Builder/Reader
```bash
cargo test cxp_builder
cargo test cxp_reader
```

## Common Patterns

### Run and Watch
```bash
cargo watch -x test
```

### Run Tests with Coverage (requires tarpaulin)
```bash
cargo tarpaulin --out Html
```

### Run Tests in Release Mode (Faster)
```bash
cargo test --release
```

## Troubleshooting

### Test Fails - Get More Info
```bash
RUST_BACKTRACE=full cargo test test_name -- --nocapture --test-threads=1
```

### Clean and Rebuild
```bash
cargo clean && cargo build && cargo test
```

### Check What Tests Exist
```bash
cargo test -- --list
```

## Test Output Interpretation

### Success
```
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured
```

### Failure
```
test result: FAILED. 34 passed; 1 failed; 0 ignored; 0 measured
```

### Ignored Tests
```
cargo test -- --ignored    # Run ignored tests
cargo test -- --include-ignored    # Run all tests including ignored
```

## Documentation

- **Full Guide**: See `TESTING.md`
- **Summary**: See `TEST_SUMMARY.md`
- **Report**: See `TEST_REPORT.md`
- **Integration Tests**: See `cxp-core/tests/README.md`
