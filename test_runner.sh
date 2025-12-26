#!/bin/bash
# CXP Test Runner Script

set -e  # Exit on error

echo "=========================================="
echo "CXP Project Test Suite"
echo "=========================================="
echo ""

# Change to project directory
cd "$(dirname "$0")"

echo "[1/4] Building project..."
cargo build --release
echo "✓ Build successful"
echo ""

echo "[2/4] Running unit tests..."
cargo test --lib
echo "✓ Unit tests passed"
echo ""

echo "[3/4] Running integration tests..."
cargo test --test integration_test
echo "✓ Integration tests passed"
echo ""

echo "[4/4] Running clippy..."
cargo clippy -- -D warnings
echo "✓ Clippy checks passed"
echo ""

echo "=========================================="
echo "All tests passed successfully!"
echo "=========================================="
