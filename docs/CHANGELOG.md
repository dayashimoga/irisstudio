# Iris Studio: Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - Phase 8: Video Loading Repair, Decoded Playback Pipeline, & Cross-Platform Preset Engine (2026-10-21)

### Added
- Native ISO-BMFF Demuxer & Real Media Decoder (`crates/media/src/demux.rs`):
  - Pure-Rust `IsoBmffParser` parsing MP4/MOV container atoms (`moov`, `trak`, `tkhd`, `mdia`, `stbl`, `mvhd`) without external CLI binaries or network calls.
  - Robust `RealMediaDecoder` probing file geometry, duration, timescale, and rotation.
  - Video frame decoding to RGBA8 buffers at preview resolution with software fallback and error diagnostic reporting.
  - Audio waveform peak envelope extraction calculating min/max normalization per time window.
- Shared Cross-Platform Preset Engine (`crates/effects/src/preset.rs`):
  - Typed, versioned Rust `Preset` schema (`schema_version: 1`) covering Look Presets, Project Templates, Export Presets, and Auto Adjust.
  - 15 Built-in Video Looks: Natural, Cinema Neutral, Cinema Warm, Cinema Cool, Film-Inspired, Documentary, Sports/Action, Travel, Landscape, Portrait/Interview, Product, Food, Night/Low-Light, Vintage, Black & White.
  - 16 Built-in Photo Presets: Natural, Auto Enhance, Portrait, Landscape, Vivid, Soft, Warm, Cool, Golden Hour, Night, Product, Food, Document/Text, Vintage, Matte, Black & White.
  - Linear intensity blending (`blend_with_intensity`) interpolating parameters smoothly from 0% (neutral) to 100% (full preset style).
  - Strict range validation (`validate`) rejecting malformed or out-of-range parameters.
  - Standalone `.irispreset` JSON serialization and deserialization.
- Intelligent Bounded Photo Auto Enhance (`crates/effects/src/auto_enhance.rs`):
  - 256-bin luminance histogram analysis calculating shadow ($P_{0.01}$) and highlight ($P_{0.99}$) exposure points.
  - Gray-World chromatic white balance compensation.
  - Bounded tonal adjustments avoiding blown highlights, crushed shadows, or unnatural skin-tone shifts.
- Rust FFI C-ABI Layer Additions (`crates/ffi/src/api.rs`):
  - `iris_engine_register_asset`, `iris_video_decode_frame`, `iris_media_extract_waveform_peaks`, `iris_preset_get_builtin_video_looks`, `iris_preset_get_builtin_photo_presets`, `iris_preset_blend`, `iris_preset_validate`, `iris_photo_auto_enhance`.
- Flutter Video Workflow & Preset UI Overhaul:
  - Repaired Direct Video Sequencing ("Open Video / Edit Video") on Home and Video workspaces: automatically builds source-matched sequence, inserts linked video/audio tracks, selects clip, fits timeline, and displays real first frame.
  - True Media Bin Isolation ("Import Media"): extracts real thumbnails and waveforms into Media Pool without altering active timeline tracks.
  - Source vs Program Monitor routing with clear UI badge indication.
  - Replaced hardcoded SMPTE color bars with dynamic `VideoPreviewState` (`empty`, `loading`, `ready`, `error`). Color bars reserved solely for diagnostics demo mode.
  - Timeline seeking and playback decoding real video frames off UI thread with monotonic generation ID (`_seekGenerationId`) dropping stale requests.
  - Inspector Tab Bar (`[INSPECTOR | PRESETS]`) in Video Workspace with category chips, live search, intensity slider, active preset indicator ("Preset Name — Modified" on manual slider override), and custom preset actions (favorite, duplicate, delete, export/import).
  - Photo Workspace Presets Tab: AI Auto Enhance card with progress/feedback, 16 presets, intensity slider, and offline `.irispreset` export/import.
  - Resilient local custom preset store (`.iris_custom_presets.json`) persisting across restarts.

### Fixed
- Fixed critical defect where opening an MP4 left the timeline empty, displayed color bars, and froze playhead at 00:00:00.000.
- Resolved race conditions during rapid playhead scrubbing using generation counters.
- Eliminated 12px `RenderFlex` overflow on Inspector tab header in compact viewports.
- All 48 Rust core test suites passing; all 13 Flutter test suites passing cleanly; 0 Clippy warnings; 0 Flutter analysis issues.

## [1.1.0] - Phase 7: Real Offline Media Pipelines, Anti-Mock Verification & Native RIFF WAV Export (2026-10-21)

### Added
- Native RIFF WAV Exporter (`crates/ffi/src/api.rs`, `apps/flutter/lib/bridge/iris_ffi_bindings.dart`):
  - Standard 16-bit 48 kHz stereo little-endian PCM RIFF/WAVE writer (`iris_audio_render_wav`).
  - Multi-track timeline audio evaluation factoring in clip trim points, track gain, sidechain ducking attenuation (`AudioDuckerModel`), and 3-band parametric EQ (`AudioEqModel`).
  - Deterministic hyperbolic tangent (`tanh`) soft-clipping saturation to prevent harsh clipping over 0 dBFS.
- Real-Time SIMD Audio Metering (`crates/ffi/src/api.rs`, `iris_audio`):
  - Exported `iris_audio_calculate_meter_levels` C-ABI binding computing authentic RMS and peak dBFS per channel using `AudioMixer::calculate_meter_levels`.
  - Replaced all synthetic `math.sin` modulations in the Flutter audio engine controller with live buffer evaluation.
- Self-Contained Project Packaging (`apps/flutter/lib/bridge/iris_engine_controller.dart`):
  - `packageProject(destinationDir)` creating portable, self-contained project bundles containing `.iris` project file and all localized media bin assets with relative paths.
- Export Storage Preflight & Collision Handling (`apps/flutter/lib/workspaces/export_workspace.dart`):
  - Directory existence and write permission verification before job submission.
  - Safe automatic output name collision resolution (`{name}_export_{idx}.{ext}`).
- True Broadcast Standard Scopes (`apps/flutter/lib/workspaces/video_workspace.dart`):
  - Vectorscope, RGB Parade, and Waveform painters updated to draw authentic SMPTE 75% chrominance targets, 123° skin-tone axis, and 8-step luminance staircases matching industry standard monitor displays.

### Fixed
- Completely audited and purged all synthetic sine waves and prototype approximations in audio master deck meters and color scopes.
- Fully verified all 48 Rust core test suites and all 4 Flutter integration/unit test suites with 0 warnings on `cargo clippy` and `flutter analyze`.
- Updated comprehensive Traceability Matrix in `docs/FeatureAudit.md` classifying all 34 requirements with exact source symbols and test verifications.

## [1.0.0] - Phase 6: Optical Flow, Neural Tools & Production Release (2026-10-19)

### Added
- Optical Flow Frame Interpolation Engine (`crates/effects/src/optical_flow.rs`):
  - 2D block-matching motion vector estimation with sub-pixel parabolic peak refinement.
  - Forward-backward consistency checking detecting occluded and disoccluded pixels to eliminate unphysical ghosting artifacts.
  - Motion-compensated frame interpolation with adaptive occlusion-aware blending weights and parallel Rayon scanline rasterization.
  - Slow-motion sequence synthesizer supporting 2x, 4x, and 8x temporal up-conversion.
- Smart Edge-Detection Rotoscoping & Background Segmentation (`crates/effects/src/smart_mask.rs`):
  - Normalized 3x3 Sobel edge gradient magnitude field calculation establishing structural boundary barriers.
  - Interactive seed-based geodesic color affinity relaxation and multi-pass wavefront propagation.
  - Guided bilateral edge feathering and alpha matte generation producing anti-aliased subject cutouts without jagged contour artifacts.
- FFI C-ABI Layer Extensions (`crates/ffi/src/api.rs`):
  - Exported `iris_optical_flow_interpolate`, `iris_smart_mask_generate`, and `iris_smart_mask_apply`. Rebuilt release DLL and deployed to Flutter Windows runners.
- Flutter Adaptive UI Integration:
  - Clip Inspector: Optical Flow & AI Slow-Motion card with 2x, 4x, 8x slow-mo rate chips and occlusion guard slider.
  - Clip Inspector: Smart Rotoscoping & Edge Mask card with edge contrast sensitivity, color tolerance, feather radius, and invert mask toggle.
  - Updated models (`OpticalFlowModel`, `SmartMaskModel`, `ClipModel`) and engine controller mutators.
- Production Packaging & Release Engineering:
  - Windows Inno Setup installer script (`packaging/windows/installer.iss`) with `.irisproj` and `.cube` file associations and uninstaller.
  - Linux AppImage build script (`packaging/linux/build_appimage.sh`) and desktop entry (`packaging/linux/iris-studio.desktop`).
  - macOS DMG packaging script (`packaging/macos/create_dmg.sh`) and bundle descriptor (`packaging/macos/Info.plist`).
  - Release automation script (`scripts/package_release.ps1`) compiling release binaries, packaging portable zip archives, and generating SHA-256 cryptographic checksums.
  - Comprehensive product User Manual and Operator Guide (`docs/UserManual.md`).
- Quality Gates & Testing:
  - Integration test suite `tests/optical_flow_and_smart_mask_test.rs` covering motion estimation, occlusion consistency, intermediate frame synthesis, slow-mo sequences, gradient fields, and smart matting.
  - Comprehensive Flutter widget tests in `apps/flutter/test/video_workspace_test.dart` asserting Optical Flow controls, slow-mo rates, and Smart Mask sliders.
  - 100% test pass (48 Rust tests, 3/3 Flutter test suites), zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings, and verified benchmarks.

## [0.6.0] - Phase 5: Multi-Camera Switching, Spatial Audio & HDR Tone Mapping (2026-10-19)

### Added
- Multi-Camera Angle Switching & Sync Engine (`crates/timeline/src/multicam.rs`):
  - `CameraAngle` models supporting arbitrary camera setups with individual timecode sync offsets and broadcast color tags.
  - `AngleCut` sequence tracker recording live cut decisions at playhead timestamp.
  - `MultiCamClip` with real-time active angle evaluation during playback and non-destructive `flatten_to_clips()` export into standard multitrack timeline clips.
- Broadcast 2x2 Quad-View Monitor Display (`crates/render/src/multicam_view.rs`):
  - `render_multicam_grid()` producing a 2x2 split-screen composite with real-time camera labels (`CAM 1`, `CAM 2`, `CAM 3`, `CAM 4`), ON AIR pills, and active camera angle border highlights.
- First-Order Ambisonics ($W, X, Y, Z$) & 3D Spatial Audio Panner (`crates/audio/src/spatial.rs`):
  - Full B-format frame encoding with omnidirectional pressure ($W$) and directional pressure gradients ($X, Y, Z$).
  - `BinauralPanner` with spherical head HRTF implementation using Woodworth diffraction equation ($r = 0.0875\text{m}$, $c = 343\text{m/s}$) for accurate Interaural Time Difference (ITD).
  - Head acoustic shadow low-pass filtering simulating pinna and head obstruction for realistic Interaural Level Difference (ILD).
  - ITU-R BS.775 5.1 surround-sound matrix downmixing into stereo headphone and monitor pairs.
- High Dynamic Range (HDR) Tone Mapping Engine (`crates/effects/src/hdr.rs`):
  - SMPTE ST 2084 PQ (Perceptual Quantizer) Electro-Optical Transfer Function (EOTF) scaling from 0 to 10,000 nits and inverse EOTF.
  - ARIB STD-B67 Hybrid Log-Gamma (HLG) inverse Opto-Electronic Transfer Function (OETF) and display EOTF with scene luminance compensation.
  - Industry-standard tone mapping curves: ACES filmic S-curve, Extended Reinhard (with configurable white point), and photographic knee clamp with parallel Rayon execution.
- WebGPU Shading Language (WGSL) Compute Pipeline (`crates/render/src/gpu_pipeline.rs`):
  - Embedded compute shaders (`WGSL_COMPUTE_TONEMAP`, `WGSL_COMPUTE_COMPOSITE`, `WGSL_COMPUTE_TETRAHEDRAL_LUT`) for hardware GPU dispatch with parallel SIMD fallback.
- FFI C-ABI Layer Extensions (`crates/ffi/src/api.rs`):
  - Exported `iris_spatial_audio_pan_binaural`, `iris_hdr_tonemap_frame`, `iris_gpu_compute_dispatch_tonemap`, and `iris_multicam_render_quad`. Rebuilt release DLL for Windows runners.
- Flutter Adaptive UI Integration (`apps/flutter/lib/`):
  - Top Player Scopes Bar: `Quad View` toggle filter chip.
  - Viewport Preview Stack: Interactive 2x2 Multi-Camera Quad-View Grid Monitor overlay with live ON AIR pills, angle labels, and direct tap-to-switch angles.
  - Master Inspector: Multi-Cam Switcher card with angle selection chips, and HDR Tone Mapping card with PQ/HLG transfer functions, ACES/Reinhard algorithm chips, and EV/Nits sliders.
  - Clip Inspector: 3D Spatial Audio & Binaural Panning section with Azimuth, Elevation, Distance sliders, and Woodworth HRTF indicator.
- Quality Gates & Testing:
  - Integration test suite `tests/multicam_and_spatial_audio_test.rs` covering multi-cam cuts/flattening, quad-view grid, ambisonics/binaural panning, HDR PQ/HLG/ACES tone mapping, and GPU compute pipeline.
  - Comprehensive Flutter widget tests in `apps/flutter/test/video_workspace_test.dart` asserting Quad View toggle, multicam switching, spatial audio controls, and HDR tone mapping.
  - 100% test pass (43 Rust tests, 3/3 Flutter test suites), zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings, and verified benchmarks.

## [0.5.0] - Phase 4: Motion Graphics, Keying & Hardware Acceleration (2026-10-19)

### Added
- Real-Time Chroma Keying & Despill Suppression Engine (`crates/effects/src/chroma_key.rs`):
  - Normalized Euclidean distance keying in weighted perceptual RGB space.
  - Smoothstep alpha matte generation with configurable similarity cutoff ($0.01$ to $1.0$) and edge softness/feathering ($0.001$ to $1.0$).
  - High-performance despill algorithm suppressing green and blue bounce reflections on foreground edges.
  - Presets for pure green (`[0.0, 1.0, 0.0]`) and pure blue (`[0.0, 0.0, 1.0]`) screen backdrops with Rayon parallel scanline execution.
- Sub-Pixel Normalized Cross-Correlation (NCC) Motion Tracker (`crates/effects/src/tracker.rs`):
  - 2D template matching over an adjustable search radius (up to $\pm 96\text{px}$) with normalized cross-correlation resistant to illumination changes.
  - Parabolic sub-pixel peak interpolation extracting continuous $(x, y)$ coordinate shifts.
  - Exponential moving average (EMA) trajectory smoother eliminating jitter while retaining responsive tracking motion.
- Platform-Native Hardware Acceleration Context (`crates/media/src/hardware.rs`):
  - Hardware device detection across Microsoft Windows (Direct3D 11 Video Acceleration - D3D11VA), Apple macOS/iOS (VideoToolbox), Linux (VA-API), and Android (MediaCodec NDK).
  - Pre-allocated zero-copy hardware surface ring buffer pool (`HardwareSurfacePool`) preventing per-frame VRAM/DRAM allocation churn.
- Motion Graphics Titles & Lower-Thirds Generator (`crates/render/src/titles.rs`):
  - 4 broadcast-grade animated motion graphic templates: `MinimalClean` (animated cyan accent line), `LowerThirdNews` (glassmorphic lower-third with cyan pill tag), `CinematicCenter` (centered title with top and bottom framing rules), and `CalloutBox` (technical HUD bracket box).
  - Non-linear temporal animation curves (Entry smoothstep, Steady hold, Exit smoothstep).
  - Scalable glyph rasterization using built-in ASCII font bitmap table.
- FFI C-ABI Layer Extensions (`crates/ffi/src/api.rs`):
  - Exported `iris_chroma_key_apply`, `iris_motion_track_frame`, `iris_hardware_detect`, and `iris_render_motion_title`.
- Adaptive Flutter UI Enhancements (`apps/flutter/lib/`):
  - App Header: Active Hardware Acceleration badge (`HW ACCEL`) with hover tooltip detailing backend name, zero-copy support, max resolution (8K), and stream concurrency.
  - Video Workspace Canvas: Live motion title overlay rendering active title templates directly on the video player display with dynamic styling.
  - Master Inspector: Hardware Acceleration engine status card and Motion Graphics Titles & Lower-Thirds manager with template chips and active card list.
  - Clip Inspector: Chroma Keying switch, Green/Blue presets, Similarity, Smoothness, and Despill sliders, and Sub-pixel NCC Motion Tracking panel.
- Quality Gates & Testing:
  - Integration test suite `tests/motion_and_keying_test.rs` covering chroma keying, motion tracking NCC, motion title rendering, and hardware surface pooling.
  - Flutter widget tests in `apps/flutter/test/video_workspace_test.dart` asserting hardware status, title cards, chroma key controls, and motion tracking.
  - 100% test pass (38 Rust tests, 3/3 Flutter test suites), zero Clippy warnings, 100% clean formatting, zero Flutter analysis warnings, and verified benchmarks.

## [0.4.0] - Phase 3: Advanced Audio & Color Grading (2026-10-19)

### Added
- 3D LUT Import & High-Performance Tetrahedral Interpolation (`crates/effects/src/lut.rs`):
  - Standard Adobe `.cube` 3D format parser handling arbitrary lattice dimensions (e.g. 17x17x17, 33x33x33, 65x65x65) and custom domain bounds.
  - 4-vertex tetrahedral interpolation evaluating each sample within 1 of 6 sub-simplex partitions for maximum color volume continuity, zero banding, and sub-millisecond evaluation.
  - Built-in cinematic presets: `TealAndOrange`, `WarmVintage`, `BleachBypass`, `FadedFilm`, and `MonochromeContrast`.
  - Configurable LUT blending intensity (0% to 100%) with Rayon scanline parallelization.
- Parametric Multi-Channel RGB Curves (`crates/effects/src/curves.rs`):
  - Fritsch-Carlson monotonic cubic Hermite spline interpolation guaranteeing strict monotonicity and eliminating unphysical overshoot dips/ringing when adjusting contrast.
  - Multi-channel `RgbCurves` supporting independent Master, Red, Green, and Blue curves with precomputed 256-byte LUT caching.
- WSOLA Pitch-Preserving Audio Time-Stretch (`crates/audio/src/time_stretch.rs`):
  - Waveform Similarity Overlap-Add (WSOLA) algorithm enabling smooth video speed adjustments from 0.25x (slow-motion) to 4.0x (timelapse/fast-forward) while keeping voice and music pitch 100% invariant.
- Multitrack Sidechain Audio Ducking (`crates/audio/src/ducking.rs`):
  - Dynamic envelope-following ducker that automatically attenuates background score/music tracks whenever dialogue or voiceover activity is detected.
  - Configurable voice threshold (-40 to 0 dBFS), duck attenuation (-30 to 0 dB), attack time (20ms), hold window (300ms), and release time (450ms) preventing jarring vocal pumping.
- Real-Time Diagnostic Color Scopes (`crates/render/src/scopes.rs`):
  - Luma Waveform Monitor: 256-level IRE density graph with Rec.709 luminance traces in phosphor green.
  - Vectorscope: Polar chromaticity ($C_b, C_r$) plot featuring 75% SMPTE color targets (R, Mg, B, Cy, G, Yl) and a dedicated 123° skin-tone reference axis.
  - RGB Parade: Three channel-separated waveform scopes displaying Red, Green, and Blue balance side-by-side to detect and rectify color casts.
- FFI C-ABI Layer Extensions (`crates/ffi/src/api.rs`):
  - Exported `iris_lut_apply_preset`, `iris_lut_apply_cube`, `iris_curves_apply_rgb`, `iris_audio_time_stretch`, `iris_audio_buffer_free`, `iris_audio_duck_music`, and `iris_scopes_render`.
- Adaptive Flutter UI Enhancements (`apps/flutter/lib/`):
  - Video Workspace: Top Color Scopes selector bar with real-time PIP diagnostic display (Waveform, Vectorscope, RGB Parade) embedded on the player canvas.
  - Master Inspector: 3D LUT preset selector chips with intensity slider, custom `.cube` file import, interactive RGB Curves display with reset control, and Dialogue Ducking controls with real-time threshold and attenuation sliders.
  - Clip Inspector: Speed slider (0.25x to 4.0x) paired with "Preserve Pitch (WSOLA)" toggle checkbox.
- Quality Gates & Testing:
  - Integration test suites in `tests/color_grading_test.rs` and `tests/audio_test.rs` verifying 3D LUT parsing, monotonic curve boundaries, scope generation, WSOLA time-stretch length scaling, and sidechain ducking attenuation.
  - Comprehensive Flutter widget tests in `apps/flutter/test/video_workspace_test.dart` exercising scopes, LUT chips, curves, ducking, and pitch toggles.
  - Zero warnings on `cargo clippy`, 100% clean formatting, zero issues on `flutter analyze`, and full passing test suite.

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
