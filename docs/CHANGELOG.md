# Iris Studio: Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - Phase 1: Photo MVP (2026-10-19)

### Added
- EXIF Orientation normalization handling tags 1-8 without loss of fidelity.
- Radial lens distortion correction utilizing the Brown-Conrady $k_1, k_2$ model.
- 4-corner perspective quadrilateral warp rectifying skewed documents and trapezoidal distortions.
- Edge-preserving bilateral-approximation denoise filter smoothing flat noisy surfaces while retaining high-contrast edges.
- Levels adjustment with 256-entry precomputed lookup table (0.47 ms latency on 1080p).
- Complete retouching suite:
  - Localized red-eye desaturation with smoothstep radial falloff.
  - Clone stamp tool with adjustable radius, feather hardness, and opacity.
  - Heal brush harmonizing high-frequency texture with ambient lighting.
  - Freehand brush and eraser with path interpolation.
- Multi-layer photo composition supporting 12 Photoshop-compatible blend modes, layer opacity, and grayscale masks.
- WebP lossless & lossy encoding with container header validation.
- Batch photo export pipeline with customizable naming templates (`{name}_{preset}_{index}`), collision strategies (`RenameWithIndex`, `Overwrite`, `Skip`), and original media preservation.
- Comprehensive Flutter Photo Workspace enhancements:
  - Tool rail with dedicated subpanels for Adjust, Crop, Retouch, Layers, Presets, and Batch Export.
  - Interactive canvas overlays for brush reticles, clone source crosshairs, and live strokes.
  - Integrated Open Photo and Export Photo dialogs.
  - Persistent batch export progress reporting and status monitoring.
- Integration tests: `tests/batch_export_test.rs`, `tests/effects_test.rs` (levels, denoise, retouch, layers), `tests/photo_pipeline_test.rs` (EXIF, lens, perspective, WebP), and Flutter widget tests (`apps/flutter/test/photo_workspace_test.dart`).

## [0.1.0] - Phase 0: Feasibility & Foundation (2026-10-19)

### Added
- Complete Architectural Documentation, Requirements, Testing Strategy, and 4 ADRs.
- Core Rust Workspace structure:
  - `iris_core`: Exact `RationalTime` arithmetic, `ColorSpace`, geometry primitives, error types.
  - `iris_media`: Demuxer interface, frame containers, pixel formats (RGBA8, BGRA8, YUV420p), hardware decoder abstraction.
  - `iris_images`: Fast image decoding/encoding (JPEG, PNG, WebP), crop, rotate, flip, Lanczos3 resize, tiled processing.
  - `iris_timeline`: Multi-track timeline model, tracks, clips, transitions, keyframes, nondestructive editing operations.
  - `iris_effects`: High-performance SIMD/parallel color adjustments (exposure, contrast, WB, highlights, shadows, curves, levels, blur, unsharp mask, vignette).
  - `iris_render`: Compositor, layer blending, preview render engine, backpressure control, cancellation tokens.
  - `iris_audio`: Audio waveform peak envelope generator, volume keyframing, multi-track mixing.
  - `iris_projects`: JSON project schema v1, undo/redo command stack, atomic autosave, Write-Ahead-Log (WAL) crash recovery journal.
  - `iris_cache`: Multi-tier bounded cache (RAM frame LRU, disk proxies, waveforms).
  - `iris_export`: Export scheduler, image batch export, video stream muxing, post-export validation pass.
  - `iris_ffi`: Typed C-ABI interface for cross-platform Flutter integration.
- Adaptive Flutter Application (`apps/flutter`):
  - Unified Home, Photo, Video, and Export workspaces.
  - Interactive photo canvas with real-time adjustments and split before/after slider.
  - Multitrack video timeline with clip trimming, playhead, snapping, and waveform rendering.
  - Production export dialog with presets, codec options, and progress tracking.
- Test suites: Unit, integration, and fuzz tests.
- CI Workflow for GitHub Actions (`.github/workflows/ci.yml`).
