# Iris Studio

> **High-Performance, 100% Offline, Cross-Platform Video & Photo Creative Workstation**  
> *Engineered with Rust Stable for low-latency media processing and Flutter for an adaptive modern UI.*

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Flutter](https://img.shields.io/badge/Flutter-3.22%2B-blue.svg)](https://flutter.dev/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green.svg)](LICENSE)
[![Offline](https://img.shields.io/badge/Telemetry-Zero%20%2F%20100%25%20Offline-brightgreen.svg)](#privacy-guarantee)

---

## Overview

**Iris Studio** is a professional-grade, privacy-first media creation suite designed for photographers, videographers, visual effects artists, and content creators. It pairs a high-throughput, multi-threaded Rust media core with an ergonomic Flutter desktop & mobile interface, delivering real-time playback, non-destructive editing, and hardware-accelerated processing across desktop and mobile form factors.

### Core Principles
- **100% Offline & Private**: Zero registration, zero cloud dependencies, zero subscriptions, zero advertisements, and zero telemetry. All assets, caches, project databases, and exports remain strictly local to your device.
- **Nondestructive Editing Pipeline**: Original media files are never altered or overwritten. Every crop, grade, transition, cut, optical flow interpolation, and smart mask is recorded as a non-destructive transformation graph with full multi-level undo/redo and atomic crash recovery.
- **Unified Creative Ecosystem**: Seamless switching between **Home/Projects**, **Photo**, **Video**, and **Export** workspaces with a shared asset library and color-managed memory buffers.
- **Sub-Millisecond Preview Latency**: Rayon parallel scanlines, SSE2/AVX2/NEON vectorization, LRU RAM/disk frame caches, and background render dispatching ensure instant scrubbing and smooth J/K/L timeline playback.

---

## Workspace Tour

### 1. Home / Projects Workspace
- **Project Hub**: Create, duplicate, archive, and manage multi-track projects with thumbnail previews and disk usage stats.
- **Template Library**: Quick-start presets for 4K UHD Cinematic, Instagram Reels / TikTok 9:16 vertical video, YouTube 1080p60, High-Resolution Still Photography, and Custom timelines.
- **Project Integrity & Recovery**: Atomic WAL journaled state saves with automatic rollback and crash recovery protection.

### 2. Photo Workspace
- **Non-Destructive Raw & Raster Pipeline**: Exposure, contrast, highlights, shadows, vibrance, saturation, and color temperature balances.
- **Color Grading & 3D LUT Engine**: Real-time `.cube` 3D LUT tetrahedral interpolation and parametric tone curves.
- **Advanced Retouching Tools**:
  - Two-frequency separation (high-frequency texture / low-frequency skin tones).
  - Edge-preserving bilateral smoothing filters.
  - Unsharp masking with radius, threshold, and amount controls.
- **Diagnostic Scopes**: Real-time RGB/Luma histograms, vector scopes, and RGB waveform analysis.

### 3. Video Workspace
- **Multi-Track Non-Linear Editor**: Unlimited video, audio, overlay, and text subtitle tracks with sub-frame magnetic snapping.
- **J/K/L Transport & Trimming**: Standard keyboard transport controls (shuttle reverse, pause, shuttle forward), split clip (`S`), ripple trim, slip/slide, and blade cuts.
- **Transition Suite**: GPU-accelerated Cross Dissolve, Dip to Color, Push/Slide, Directional Wipe, and Radial Zoom transitions.
- **Optical Flow Frame Interpolation (AI Slow-Mo)**:
  - Bidirectional block-matching motion vector estimation with sub-pixel parabolic peak refinement.
  - Forward-backward consistency checking for occlusion and disocclusion detection without ghosting.
  - 2x, 4x, and 8x temporal up-conversion and cinematic slow-motion synthesis.
- **Smart Edge-Detection Rotoscoping & Background Segmentation**:
  - Sobel edge gradient magnitude field barrier calculations.
  - Seed-based geodesic color affinity relaxation and multi-pass wavefront propagation.
  - Bilateral edge feathering and alpha matte background cutouts.
- **Keyframe Engine & Automation**: Bezier and linear spatial/temporal keyframing for position, scale, rotation, opacity, and audio gain.
- **Integrated Audio Mixer**: Track volume sliders, pan pots, solo/mute toggles, 10-band graphic equalizer, peak limiter, and stereo decibel peak meters.

### 4. Export Workspace
- **Multi-Format Export Engine**: Render to MP4 (H.264 / H.265), WebM (VP9 / Opus), Apple ProRes 422/4444, and uncompressed TIFF / PNG / JPEG image sequences.
- **Preconfigured Delivery Presets**: YouTube 4K, Instagram Reels (1080x1920), Cinema DCI 4K (24.0 fps), Audio Only (WAV/FLAC), and Archival Masters.
- **Background Render Queue**: Asynchronous batch job queuing, pause/resume, and real-time progress, time-remaining, and ETA estimation.

---

## Architecture

Iris Studio is designed as a modular, decoupled workspace consisting of Rust engine crates, an FFI boundary, and an adaptive Flutter application:

```
rustvideoaudio/
├── crates/
│   ├── core/           # Memory buffers, pixel formats, timeline model, color transforms
│   ├── storage/        # Project serialization, undo/redo, autosave WAL journals
│   ├── playback/       # Audio/video synchronization, transport controllers, frame cache
│   ├── effects/        # Color grading, 3D LUT, optical flow, smart mask, transitions
│   ├── export/         # Transcoding pipeline, encoders, multiplexers, batch queues
│   ├── audio/          # Mixing engine, 10-band EQ, compressor/limiter, spatial panning
│   └── ffi/            # C-ABI export surface and dynamic library interface
├── apps/flutter/       # Adaptive Flutter UI (Desktop Windows/macOS/Linux, Mobile iOS/Android)
├── benchmarks/         # Standardized micro & macro performance benchmark suite
├── packaging/          # Inno Setup (.iss), AppImage, macOS DMG, and Linux desktop files
├── docs/               # System architecture, ADRs, user manuals, and technical specs
└── scripts/            # Cross-platform test, build, and packaging automation
```

---

## Getting Started

### Prerequisites
- **Rust**: Version 1.80.0 or higher (`rustup default stable`)
- **Flutter SDK**: Version 3.22.0 or higher
- **C/C++ Compiler & CMake**: MSVC (Windows), GCC/Clang (Linux), or Xcode Command Line Tools (macOS)
- **PowerShell** (Windows) or **Bash** (Linux/macOS)

### Building the Native Core
```bash
# Clone the repository
git clone https://github.com/your-org/iris-studio.git
cd iris-studio

# Build optimized release FFI library
cargo build --release -p iris_ffi

# Run all Rust workspace unit and integration tests (48 tests)
cargo test --workspace

# Run strict clippy verification
cargo clippy --workspace --all-targets -- -D warnings
```

### Running Performance Benchmarks
```bash
cargo run --release --bin run_benchmarks
```

### Launching the Desktop UI
```bash
# Copy compiled iris_ffi library to Flutter directory
# Windows:
copy target\release\iris_ffi.dll apps\flutter\
copy target\release\iris_ffi.dll apps\flutter\windows\runner\

# Run the Flutter desktop app
cd apps/flutter
flutter pub get
flutter test
flutter run -d windows # Or: -d macos, -d linux
```

---

## Distribution & Packaging

Iris Studio provides automated scripts to create platform-native distribution packages:

- **Windows Installer & Portable ZIP**:
  ```powershell
  powershell.exe -ExecutionPolicy Bypass -File scripts/package_release.ps1
  ```
  Generates `dist/windows/IrisStudio-v1.0.0-windows-x64.zip` and calculates cryptographic `SHA256SUMS.txt`. An Inno Setup script is available at `packaging/windows/installer.iss`.

- **Linux AppImage**:
  ```bash
  bash packaging/linux/build_appimage.sh
  ```

- **macOS DMG**:
  ```bash
  bash packaging/macos/create_dmg.sh
  ```

---

## Privacy Guarantee

Iris Studio adheres to the strictest privacy and local-first software engineering principles:
- **No Network Sockets**: The core editing engine does not open network listeners or outbound client sockets.
- **No Analytics / Telemetry**: No tracking identifiers, error reporting telemetry, or usage metrics are transmitted.
- **Local File Security**: Your media files, projects, undo histories, and render outputs never leave your storage drive.

---

## Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:
- [**User Manual**](docs/UserManual.md): In-depth guide covering all workspaces, keyboard shortcuts, and editing workflows.
- [**Architecture Guide**](docs/Architecture.md): Detailed subsystem designs, threading models, and FFI bridges.
- [**Codec Compatibility**](docs/CodecCompatibility.md): Supported containers, pixel formats, and hardware decoders.
- [**Changelog**](docs/CHANGELOG.md): Complete release history from Phase 0 to Phase 6 [1.0.0].

---

## License

Iris Studio is dual-licensed under either:
- **MIT License** ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
