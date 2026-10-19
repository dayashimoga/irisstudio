#!/usr/bin/env bash
# Iris Studio - macOS DMG Creation Script
set -euo pipefail

APP_NAME="Iris Studio"
VERSION="1.0.0"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist/macos"
DMG_STAGING="${DIST_DIR}/staging"
DMG_OUTPUT="${DIST_DIR}/IrisStudio-${VERSION}-Universal.dmg"

echo "==> Building Iris Studio Rust FFI Universal Binary..."
cargo build --release -p iris_ffi --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Building Flutter macOS App Bundle..."
cd "${ROOT_DIR}/apps/flutter"
flutter build macos --release

echo "==> Preparing DMG Staging Directory..."
rm -rf "${DMG_STAGING}" "${DMG_OUTPUT}"
mkdir -p "${DMG_STAGING}"

cp -R "${ROOT_DIR}/apps/flutter/build/macos/Build/Products/Release/iris_studio.app" "${DMG_STAGING}/${APP_NAME}.app"
cp "${ROOT_DIR}/target/release/libiris_ffi.dylib" "${DMG_STAGING}/${APP_NAME}.app/Contents/Frameworks/" || true
ln -s /Applications "${DMG_STAGING}/Applications"

echo "==> Creating DMG Disk Image..."
if command -v hdiutil &> /dev/null; then
    hdiutil create -volname "${APP_NAME}" -srcfolder "${DMG_STAGING}" -ov -format UDZO "${DMG_OUTPUT}"
    echo "==> DMG successfully created at ${DMG_OUTPUT}"
else
    echo "==> [NOTICE] hdiutil is only available on macOS hosts. App bundle staged at ${DMG_STAGING}."
fi
