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

## Phase 4: Motion Graphics, Keying & Hardware Acceleration (Completed & Verified)
- [x] Chroma Keying (Green / Blue screen) with despill correction, Euclidean color distance, and smoothstep alpha feathering (`crates/effects/src/chroma_key.rs`).
- [x] Motion tracking engine with 2D Normalized Cross-Correlation (NCC), sub-pixel parabolic peak refinement, and exponential moving average trajectory smoothing (`crates/effects/src/tracker.rs`).
- [x] Platform hardware acceleration context and zero-copy surface ring buffer pool abstraction across Windows (D3D11VA), macOS/iOS (VideoToolbox), Linux (VA-API), and Android (MediaCodec) (`crates/media/src/hardware.rs`).
- [x] Motion graphics title cards and lower-thirds generator with 4 animated templates (`MinimalClean`, `LowerThirdNews`, `CinematicCenter`, `CalloutBox`) and smooth temporal easing (`crates/render/src/titles.rs`).
- [x] FFI C-ABI interface additions (`iris_chroma_key_apply`, `iris_motion_track_frame`, `iris_hardware_detect`, `iris_render_motion_title`).
- [x] Adaptive Flutter UI integration:
  - [x] App Header: Dynamic Hardware Acceleration badge (`HW ACCEL`) with device capability tooltip.
  - [x] Video Workspace: Active motion title card live overlay on video player canvas, Master Inspector Hardware Acceleration Card, and Motion Titles & Lower-Thirds management.
  - [x] Clip Inspector: Chroma Keying toggle, Green/Blue presets, Similarity, Smoothness, and Despill sliders, and Sub-pixel NCC Motion Tracking inspector.
- [x] Comprehensive test suites (`tests/motion_and_keying_test.rs`, `apps/flutter/test/video_workspace_test.dart`) and verified quality gates (38 Rust tests pass, zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings, 3/3 Flutter test suites pass).

## Phase 5: Multi-Camera Switching & Spatial Audio (Completed & Verified)
- [x] Multi-camera synchronized angle ingest and real-time live angle switching engine (`crates/timeline/src/multicam.rs`):
  - [x] `CameraAngle` models with per-angle sync offsets, color tags, and source asset linkage.
  - [x] `MultiCamClip` supporting live angle switching, angle cuts recording at timeline playhead, and non-destructive `flatten_to_clips()` export into standard timeline clips.
- [x] Ambisonic B-format & Binaural 3D Spatial Audio Panner (`crates/audio/src/spatial.rs`):
  - [x] First-Order Ambisonics ($W, X, Y, Z$) B-format encoding for omnidirectional and directional audio pressure fields.
  - [x] `BinauralPanner` with spherical head HRTF model based on Woodworth head diffraction ($r = 0.0875\text{m}$, $c = 343\text{m/s}$) for accurate Interaural Time Difference (ITD) and head acoustic shadow filtering for Interaural Level Difference (ILD).
  - [x] ITU-R BS.775 5.1 surround downmixing matrix into stereo pair.
- [x] HDR (High Dynamic Range) Tone Mapping & Transfer Functions (`crates/effects/src/hdr.rs`):
  - [x] SMPTE ST 2084 PQ (Perceptual Quantizer) EOTF ($0 \to 10000$ nits) and inverse EOTF.
  - [x] ARIB STD-B67 HLG (Hybrid Log-Gamma) inverse OETF and display EOTF.
  - [x] Industry-standard tone mapping curves: ACES filmic S-curve, Extended Reinhard (with adjustable white point), and photographic knee clamp with parallel Rayon execution.
- [x] GPU-Accelerated Compute Pipeline (`crates/render/src/gpu_pipeline.rs`, `crates/render/src/multicam_view.rs`):
  - [x] Embedded WGSL compute shaders (`WGSL_COMPUTE_TONEMAP`, `WGSL_COMPUTE_COMPOSITE`, `WGSL_COMPUTE_TETRAHEDRAL_LUT`) for real-time GPU dispatch with multi-threaded parallel fallback.
  - [x] Broadcast 2x2 Quad-View Monitor renderer (`render_multicam_grid`) displaying 4 synchronized angle viewports with on-air status pills and active border highlights.
- [x] FFI C-ABI Layer additions (`crates/ffi/src/api.rs`):
  - [x] Exported `iris_spatial_audio_pan_binaural`, `iris_hdr_tonemap_frame`, `iris_gpu_compute_dispatch_tonemap`, and `iris_multicam_render_quad`.
- [x] Flutter Adaptive UI integration (`apps/flutter/`):
  - [x] Video Scopes bar: `Quad View` toggle filter chip.
  - [x] Viewport Canvas: Interactive 2x2 Quad-View Grid Monitor overlay with live ON AIR pills, angle labels, and direct tap-to-switch angles.
  - [x] Master Inspector: Multi-Cam Switcher card with angle selection chips, and HDR Tone Mapping card with PQ/HLG transfer functions, ACES/Reinhard algorithm chips, and EV/Nits sliders.
  - [x] Clip Inspector: 3D Spatial Audio & Binaural Panning section with Azimuth, Elevation, Distance sliders, and Woodworth HRTF indicator.
- [x] Comprehensive test suites (`tests/multicam_and_spatial_audio_test.rs`, `apps/flutter/test/video_workspace_test.dart`) and verified quality gates (43 Rust tests pass, zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings, 3/3 Flutter test suites pass).

## Phase 6: Optical Flow, Neural Tools & Final Release Polish (Completed & Verified)
- [x] Optical Flow Frame Interpolation Engine (`crates/effects/src/optical_flow.rs`):
  - [x] Bidirectional motion vector estimation (`estimate_motion_field`) with block matching and sub-pixel parabolic peak refinement.
  - [x] Forward-backward consistency checking (`check_forward_backward_consistency`) for robust occlusion detection without ghosting.
  - [x] Motion-compensated frame interpolation (`interpolate_frame`) with adaptive occlusion weighting and parallel Rayon scanline execution.
  - [x] Slow-motion sequence generator (`generate_slow_motion_sequence`) supporting 2x, 4x, and 8x fluid temporal up-conversion.
- [x] Smart Edge-Detection Rotoscoping & Background Segmentation Engine (`crates/effects/src/smart_mask.rs`):
  - [x] Contrast-aware 3x3 Sobel edge gradient magnitude field calculation.
  - [x] Seed-based geodesic color affinity relaxation and boundary barrier propagation.
  - [x] Guided bilateral feathering and alpha matte generation (`generate_smart_mask`, `apply_smart_mask`).
- [x] FFI C-ABI Layer Additions (`crates/ffi/src/api.rs`):
  - [x] Exported `iris_optical_flow_interpolate`, `iris_smart_mask_generate`, and `iris_smart_mask_apply`. Rebuilt release DLL for Windows runners.
- [x] Flutter Adaptive UI Integration:
  - [x] Clip Inspector: Optical Flow & AI Slow-Motion card (2x, 4x, 8x chips, occlusion guard slider).
  - [x] Clip Inspector: Smart Rotoscoping & Edge Mask card (contrast sensitivity, color tolerance, feather radius, invert mask switch).
  - [x] Flutter model updates (`OpticalFlowModel`, `SmartMaskModel`, `ClipModel`) and engine controller mutators.
- [x] Production Packaging & Release Engineering:
  - [x] Windows Inno Setup script (`packaging/windows/installer.iss`) with `.irisproj` and `.cube` file associations and uninstaller.
  - [x] Linux AppImage build script (`packaging/linux/build_appimage.sh`) and desktop entry (`packaging/linux/iris-studio.desktop`).
  - [x] macOS DMG packaging script (`packaging/macos/create_dmg.sh`) and bundle descriptor (`packaging/macos/Info.plist`).
  - [x] Release automation script (`scripts/package_release.ps1`) compiling release binaries, creating distribution bundles, and computing cryptographic SHA-256 checksums.
  - [x] Comprehensive product User Manual and Operator Guide (`docs/UserManual.md`).
- [x] Comprehensive Quality Gates:
  - [x] 48 Rust unit & integration tests passing (`tests/optical_flow_and_smart_mask_test.rs`).
  - [x] Clippy zero-warnings gate: 0 warnings on `cargo clippy --workspace --all-targets -- -D warnings`.
  - [x] 100% clean formatting on `cargo fmt --all -- --check`.
  - [x] Flutter analyzer: 0 issues on `flutter analyze`.
  - [x] Flutter widget tests: 3/3 test suites passing (`video_workspace_test.dart`, `photo_workspace_test.dart`, `widget_test.dart`).





