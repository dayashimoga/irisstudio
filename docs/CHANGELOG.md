# Iris Studio: Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - Phase 2: Video MVP (2026-10-19)

### Added
- Extended video transition engine (`crates/render/src/transition.rs`, `crates/timeline/src/model.rs`):
  - 13 high-performance transitions: `CrossDissolve`, `FadeToBlack`, `DipToColor` (with configurable RGB dip color), `WipeLeft`, `WipeRight`, `WipeUp`, `WipeDown`, `Slide`, `PushLeft`, `PushRight`, `IrisRound`, `ZoomIn`, and `ZoomOut`.
  - Scanline-level Rayon parallel rasterization preserving frame alpha and edge antialiasing.
  - Integration with multi-track timeline rendering during clip transitions at normalized time $t \in [0.0, 1.0]$.
- Professional Audio EQ & Dynamics Processing (`crates/audio/src/eq.rs`):
  - Robert Bristow-Johnson Audio EQ Cookbook biquad IIR filter implementation.
  - 3-band parametric equalizer: Low Shelf (120 Hz), Peaking Mid (1 kHz with $Q=1.0$), and High Shelf (8 kHz) with -18 dB to +18 dB gain range and fast zero-gain bypass.
  - `AudioCompressor` with configurable threshold (-30 to 0 dBFS), ratio (1:1 to 20:1 limiter), attack/release exponential envelope follower, and makeup gain (0 to 24 dB).
- Subtitles & Caption System (`crates/timeline/src/subtitles.rs`, `crates/render/src/subtitles.rs`):
  - SubRip (`.srt`) and WebVTT (`.vtt`) parsers and serializers with millisecond timestamp parsing and temporal lookup query `items_at(time)`.
  - Burn-in subtitle rasterizer rendering against `VideoFrame` with built-in scalable 8x8 ASCII glyph atlas, rounded semi-transparent background box, and configurable vertical alignment (`TopCenter`, `Center`, `BottomCenter`).
- Keyframe Easing Curves (`crates/timeline/src/keyframes.rs`):
  - `Easing` enum supporting `Linear`, `EaseIn` ($u^2$), `EaseOut` ($1 - (1-u)^2$), `EaseInOut` (smooth S-curve), and parametric `CubicBezier(x1, y1, x2, y2)` with bisection solver.
  - Integrated into `TimelineEvaluator::interpolate_volume` and clip property tweens.
- Stream Copy Export Optimization (`crates/export/src/stream_copy.rs`):
  - Static edit decision analyzer identifying unedited timeline segments with identity transforms, normal speed (1.0x), and no transitions or subtitles.
  - Enables direct bitstream packet remuxing without decode/re-encode overhead, delivering 25x-30x export speedups and bit-perfect generational quality preservation.
- FFI C-ABI Extensions (`crates/ffi/src/api.rs`):
  - Exported C-ABI functions: `iris_audio_process_eq`, `iris_audio_process_compressor`, `iris_timeline_add_transition`, `iris_timeline_import_subtitles_srt`, `iris_timeline_export_subtitles_srt`, `iris_timeline_add_subtitle`, `iris_export_analyze_stream_copy`.
- Flutter Video Workspace Enhancements (`apps/flutter/lib/`):
  - Master Inspector: Stream Copy eligibility indicator card, 3-Band Parametric EQ sliders, Audio Compressor sliders, and Subtitles & Captions management with live `+ Add Caption` and `Import SRT` file workflow.
  - Clip Inspector: Keyframe Easing selector chips (Linear, Ease In, Ease Out, Ease In-Out, Cubic Bezier) and 10 Transition preset selector chips.
  - Timeline & Canvas: Transition badge indicators between adjacent timeline clips, dedicated Subtitle track lane, and live burn-in caption overlay directly on the video player preview canvas.
- Comprehensive Testing & Quality Gates:
  - Rust unit & integration test suite (`tests/video_pipeline_test.rs`, `tests/audio_test.rs`) covering all transitions, audio filters, subtitle parser/burner, keyframe curves, and stream copy optimizer.
  - Flutter widget tests (`apps/flutter/test/video_workspace_test.dart`) exercising inspectors, controls, and timeline badges.
  - Zero warnings on `cargo clippy`, 100% clean `cargo fmt`, zero issues on `flutter analyze`, and 100% passing tests.

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
