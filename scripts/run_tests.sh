#!/bin/bash

# Local test runner script for ProofZK
# Run this before pushing to ensure CI will pass

set -e

echo "Running ProofZK Test Suite"
echo "================================"

echo "Checking code formatting..."
cargo fmt --all -- --check

echo "Running Clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings

echo "Running unit tests..."
cargo test --workspace --verbose

echo "Running tests in release mode..."
cargo test --workspace --release

echo "Testing documentation..."
cargo test --doc --workspace

echo "Building documentation..."
cargo doc --workspace --no-deps --document-private-items

echo "Running benchmarks..."
cd core && cargo bench

echo ""
echo "All tests passed! Ready to push to GitHub."
echo "GitHub Actions will run the same checks on PR/merge."