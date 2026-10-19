# Iris Studio: Master Task List & Roadmap

## Phase 0: Feasibility & Foundation (Completed)
- [x] Create comprehensive architectural documentation, ADRs, matrices, and governance.
- [x] Establish Cargo workspace and modular crate boundaries.
- [x] Implement foundational core crates:
  - [x] `iris_core`: Rational time, color space, geometry, errors.
  - [x] `iris_media`: Demuxer, frame types, hardware decode abstraction.
  - [x] `iris_images`: Image transforms, codecs, crop/rotate/straighten, tiled processing.
  - [x] `iris_timeline`: Tracks, clips, transitions, keyframes, nondestructive edits.
  - [x] `iris_effects`: Exposure, contrast, white balance, curves, levels, blur, sharpening, vignette.
  - [x] `iris_render`: Compositing engine, layer blending, double-buffering, backpressure.
  - [x] `iris_audio`: Waveforms, mixing, volume envelopes, stereo metering.
  - [x] `iris_projects`: JSON schema, undo/redo, atomic autosave, WAL crash journal.
  - [x] `iris_cache`: Bounded LRU cache, memory budgeting, proxy management.
  - [x] `iris_export`: Range export, batch photo processing, validation pass.
  - [x] `iris_ffi`: Typed C-ABI interface for Flutter.
- [x] Implement Flutter Adaptive Application (`apps/flutter`):
  - [x] Modern UI design system (dark aesthetic, clean typography, responsive layout).
  - [x] Home / Projects workspace.
  - [x] Photo workspace with interactive canvas, before/after slider, and parametric controls.
  - [x] Video workspace with multitrack timeline, playhead, snapping, and waveform display.
  - [x] Export workspace with presets, codec configuration, and progress display.
  - [x] FFI Bridge linking Dart to Rust engine with fallback modes.
- [x] Create unit, integration, and property tests.
- [x] Setup CI workflow (`.github/workflows/ci.yml`).
- [x] Validate Phase 0 verification gates.

## Phase 1: Photo MVP (Completed & Verified)
- [x] EXIF orientation normalization (tags 1-8 lossless transformations).
- [x] Radial lens distortion correction (Brown-Conrady k1, k2 model).
- [x] Perspective 4-corner quad warp and rectifying homography.
- [x] Retouching tools: red-eye removal, clone stamp, heal brush, freehand brush/eraser.
- [x] Edge-preserving bilateral denoise filter.
- [x] Levels adjustments with high-performance 256-entry precomputed LUT.
- [x] Multi-layer photo composition with 12 blend modes and grayscale opacity masks.
- [x] WebP lossless & lossy encoding/decoding and format validation.
- [x] Batch image export with naming templates (`{name}_{preset}_{index}`), collision rules (`RenameWithIndex`, `Overwrite`, `Skip`), and original preservation.
- [x] Adaptive Flutter Photo workspace with interactive retouching cursors, before/after slider, layer manager, and batch export progress.
- [x] Rust integration tests (`photo_pipeline_test`, `effects_test`, `batch_export_test`) and Flutter widget tests (`photo_workspace_test`).
- [x] Verified verification gates (100% test pass, zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings).

## Phase 2: Video MVP (Completed & Verified)
- [x] Extended transition presets (13 transitions: CrossDissolve, FadeToBlack, DipToColor, WipeLeft/Right/Up/Down, Slide, PushLeft/Right, IrisRound, ZoomIn/Out) with Rayon parallel scanline rasterization.
- [x] Robert Bristow-Johnson Audio EQ Cookbook 3-band parametric equalizer (Low Shelf, Peaking Mid, High Shelf) with zero-gain bypass.
- [x] AudioCompressor with configurable threshold (-30 to 0 dBFS), ratio (1:1 to 20:1), attack/release exponential envelope follower, and makeup gain (0 to 24 dB).
- [x] Subtitles and Caption engine with SubRip (.srt) and WebVTT (.vtt) parsers, serializers, and crisp burn-in overlay with built-in ASCII glyph atlas.
- [x] Keyframe Easing curves (Linear, EaseIn, EaseOut, EaseInOut, CubicBezier with bisection solver) wired to volume envelopes and parametric transforms.
- [x] Stream copy optimization (`StreamCopyOptimizer`) for unedited timeline cuts enabling 25x-30x export acceleration with zero generational quality loss.
- [x] Adaptive Flutter Video workspace with Master Inspector (Stream Copy status, 3-Band EQ, Compressor, Subtitles & Captions with SRT import), Clip Inspector (Transition selector chips, Keyframe Easing curve chips), multitrack transition badges, and live subtitle burn-in preview.
- [x] Full integration test suites (`video_pipeline_test.rs`, `audio_test.rs`) and Flutter widget test suite (`video_workspace_test.dart`).
- [x] Verified quality gates (100% test pass, zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings).

## Phase 3: Advanced Audio & Color Grading (Completed & Verified)
- [x] 3D LUT parser and engine supporting standard Adobe `.cube` formats with 4-vertex tetrahedral interpolation and 5 cinematic presets (`TealAndOrange`, `WarmVintage`, `BleachBypass`, `FadedFilm`, `MonochromeContrast`).
- [x] Parametric RGB Curves with Fritsch-Carlson monotonic cubic spline evaluation preventing overshoot and color ringing.
- [x] WSOLA (Waveform Similarity Overlap-Add) audio time-stretch algorithm for speed variations (0.25x - 4.0x) with 100% pitch preservation.
- [x] Multitrack sidechain audio ducking (`AudioDucker`) triggered by dialogue tracks with configurable threshold, attenuation, attack, hold, and release.
- [x] Diagnostic color scopes: Luma Waveform Monitor, Vectorscope (with 75% SMPTE targets and 123° skin-tone axis), and RGB Parade.
- [x] FFI C-ABI interface additions (`iris_lut_apply_preset`, `iris_lut_apply_cube`, `iris_curves_apply_rgb`, `iris_audio_time_stretch`, `iris_audio_duck_music`, `iris_scopes_render`).
- [x] Flutter adaptive UI integration: Color Scopes bar and canvas PIP overlay, 3D LUT preset selector chips with intensity slider, parametric curves preview, audio ducking controls, and clip speed slider with WSOLA pitch preservation toggle.
- [x] Comprehensive test suites (`color_grading_test.rs`, `audio_test.rs`, `video_workspace_test.dart`) and verified quality gates (34 Rust tests pass, zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings, 3/3 Flutter test suites pass).

## Phase 4: Motion Graphics, Keying & Hardware Acceleration (Next Phase)
- [ ] Chroma Keying (Green / Blue screen) with despill correction and soft edge feathering.
- [ ] Motion tracking engine (optical flow / template cross-correlation) for sticky overlays and titles.
- [ ] Platform hardware acceleration wrappers (DirectX 11/12 DXVA on Windows, VideoToolbox on macOS/iOS, VA-API on Linux, MediaCodec on Android).
- [ ] Motion Title Card and Lower-Thirds generator with animated text templates.


