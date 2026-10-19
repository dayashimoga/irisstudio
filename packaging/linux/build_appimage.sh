#!/usr/bin/env bash
# Iris Studio - Linux AppImage Build Script
set -euo pipefail

APP_NAME="iris-studio"
VERSION="1.0.0"
ARCH=$(uname -m)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD_DIR="${ROOT_DIR}/dist/linux"
APP_DIR="${BUILD_DIR}/AppDir"

echo "==> Building Iris Studio Rust FFI Release Library..."
cargo build --release -p iris_ffi --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Building Flutter Linux Bundle..."
cd "${ROOT_DIR}/apps/flutter"
flutter build linux --release

echo "==> Assembling AppDir Structure..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"
mkdir -p "${APP_DIR}/usr/lib"
mkdir -p "${APP_DIR}/usr/share/applications"
mkdir -p "${APP_DIR}/usr/share/icons/hicolor/256x256/apps"

cp -r "${ROOT_DIR}/apps/flutter/build/linux/x64/release/bundle/"* "${APP_DIR}/usr/bin/"
cp "${ROOT_DIR}/target/release/libiris_ffi.so" "${APP_DIR}/usr/lib/" || true
cp "${ROOT_DIR}/packaging/linux/iris-studio.desktop" "${APP_DIR}/usr/share/applications/"
cp "${ROOT_DIR}/packaging/linux/iris-studio.desktop" "${APP_DIR}/"

cat << 'EOF' > "${APP_DIR}/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${HERE}/usr/bin/lib:${LD_LIBRARY_PATH:-}"
exec "${HERE}/usr/bin/iris_studio" "$@"
EOF
chmod +x "${APP_DIR}/AppRun"

echo "==> Packaging AppImage via appimagetool..."
if command -v appimagetool &> /dev/null; then
    appimagetool "${APP_DIR}" "${BUILD_DIR}/IrisStudio-${VERSION}-${ARCH}.AppImage"
    echo "==> Linux AppImage created at ${BUILD_DIR}/IrisStudio-${VERSION}-${ARCH}.AppImage"
else
    echo "==> [NOTICE] appimagetool not found in PATH. AppDir prepared at ${APP_DIR}."
fi
