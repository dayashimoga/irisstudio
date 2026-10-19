#!/usr/bin/env bash
set -euo pipefail

echo "==> Running Cargo Test Suite across workspace..."
cargo test --workspace

echo "==> Verifying Clippy Zero Warnings Gate..."
cargo clippy --workspace --all-targets -- -D warnings

echo "==> Verifying Rust Formatting..."
cargo fmt --all -- --check

echo "==> Verifying Flutter Analysis & Tests..."
cd apps/flutter
flutter analyze
flutter test
cd ../..

echo "==> Running Performance Benchmarks..."
cargo run --release --bin run_benchmarks

echo "==> ALL QUALITY AND VERIFICATION GATES PASSED!"
