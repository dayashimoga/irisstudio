# Iris Studio: Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
