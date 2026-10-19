#!/usr/bin/env bash
set -euo pipefail

echo "==> Building Iris Studio Rust Core Engine (Release)..."
cargo build --release --workspace

echo "==> Deploying shared library to Flutter app..."
mkdir -p apps/flutter/linux/
cp -f target/release/libiris_ffi.* apps/flutter/ || true

echo "==> Resolving Flutter dependencies..."
cd apps/flutter
flutter pub get

echo "==> Build complete successfully!"
