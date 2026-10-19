# Iris Studio: Environment Setup & Toolchain Guide

## 1. Prerequisites

### 1.1 Core Toolchains
- **Rust Stable** (1.80+):
  ```bash
  # Verify installation
  rustc --version
  cargo --version
  ```
- **Flutter SDK** (3.24+):
  ```bash
  flutter --version
  flutter doctor
  ```
- **C/C++ Build Tools**:
  - Windows: MSVC C++ Build Tools or MinGW-W64 GCC / Clang.
  - Linux: `build-essential`, `pkg-config`, `libasound2-dev`, `libva-dev`.
  - macOS: Xcode Command Line Tools.

---

## 2. Workspace Initialization

Clone the repository and verify the environment:
```bash
git clone https://github.com/iris-studio/iris-studio.git
cd iris-studio
```

Compile the Rust engine workspace:
```bash
cargo build --workspace
cargo test --workspace
```

Prepare the Flutter application:
```bash
cd apps/flutter
flutter pub get
flutter analyze
```

---

## 3. Running Iris Studio Locally

### Desktop (Windows, macOS, Linux)
```bash
# From workspace root
cargo build --release -p iris_ffi
# Launch Flutter application
cd apps/flutter
flutter run -d windows    # or -d macos, -d linux
```

### Mobile (Android & iOS)
```bash
# Build Android native libraries
cargo build --target aarch64-linux-android --release -p iris_ffi
cd apps/flutter
flutter run -d <device_id>
```
