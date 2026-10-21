# Iris Studio: Testing Strategy

## 1. Quality Standards & Coverage Goals
- **Domain & Engine Code Coverage**: Minimum 90% branch/statement coverage for `iris_core`, `iris_timeline`, `iris_projects`, `iris_effects`, `iris_images`, and `iris_render`.
- **Zero Warnings Policy**: `cargo clippy --workspace --all-targets -- -D warnings` must produce zero warnings.
- **Flutter Analysis**: `flutter analyze` in `apps/flutter` must pass cleanly without info warnings.

---

## 2. Test Pyramid & Execution

### 2.1 Unit Tests (Rust)
- Pure deterministic logic:
  - Rational time math (`RationalTime::add`, `rescale`, `to_seconds`).
  - Nondestructive edit commands (`TrimCommand`, `SplitCommand`, `RippleDeleteCommand`).
  - Project JSON serialization and deserialization round-trip.
  - Pixel math accuracy: exposure formula, gamma conversion, Gaussian convolution matrix, unsharp masking.

### 2.2 Integration Tests (`/tests`)
- `tests/photo_pipeline_test.rs`: Loads test image, applies exposure, contrast, crop, and verifies output histogram and dimensions.
- `tests/timeline_composition_test.rs`: Assembles 3-track timeline with overlapping transition, evaluates frame at $t=1.5s$, verifies alpha blend.
- `tests/crash_recovery_test.rs`: Writes 10 commands to journal, simulates abrupt termination, re-opens project, verifies 100% state restoration.
- `tests/audio_waveform_test.rs`: Generates peak envelopes from synthetic sine/noise buffers, verifies min/max peak invariants.

### 2.3 Property-Based Testing
- Using `proptest` on timeline manipulation:
  - Any sequence of random splits, trims, and moves must maintain invariant that clips on a single track never overlap in timeline coordinates.

### 2.4 Fuzz Testing
- Fuzzing image decoders and project deserializers against malformed byte streams and decompression bombs.

### 2.5 Video Pipeline, Real Demuxer & Preset Engine Tests (`/tests`)
- `tests/video_pipeline_test.rs`:
  - `test_iso_bmff_mp4_probing`: Validates pure-Rust parsing of ISO-BMFF atoms (`moov`, `trak`, `tkhd`, `mdia`, `stbl`, `mvhd`) across real 360p, vertical 1080x1920, and audio-less fixtures.
  - `test_real_media_decoder_video_frame`: Asserts native video decoding to RGBA8 buffer with correct resolution, stride, and content.
  - `test_real_media_decoder_audio_waveform`: Verifies audio sample demuxing, RMS/peak extraction, and normalized peak envelope invariance.
  - `test_render_timeline_frame_with_preset`: Applies Video Look Presets (e.g. Cinema Warm, Vintage) to timeline composition and verifies color grade matrix modifications.
  - `test_preset_intensity_blending`: Tests linear parameter blending at 0%, 50%, and 100% intensity, ensuring neutral values at 0% and target look values at 100%.
  - `test_preset_schema_validation`: Validates range constraints (exposure, contrast, saturation, temperature, tint, highlights, shadows, vignette) and rejection of malformed or out-of-bounds presets.
  - `test_photo_auto_enhance_bounded_analysis`: Verifies that luminance percentile analysis and Gray-World white balance remain within strict aesthetic bounds.
  - `test_preset_json_roundtrip_offline`: Validates serialization and deserialization of `.irispreset` files without external dependencies or cloud connectivity.

---

## 3. Flutter UI Automation & Integration Tests (`apps/flutter/test`)

- `test/video_loading_and_presets_test.dart`:
  - **Direct Video Sequencing**: Opening a video file automatically creates a source-matched sequence, inserts linked video/audio tracks, selects the clip, fits the timeline, decodes the real first frame, and transitions `VideoPreviewState` to `ready`.
  - **Media Bin Isolation**: Importing a media asset into the bin generates thumbnails and waveform peaks without modifying or overwriting existing timeline sequences.
  - **Source vs Program Preview**: Validates that selecting a bin asset activates Source Preview without altering the current Program timeline playhead.
  - **Look Preset Application & Intensity**: Tests instant preset application, intensity slider updates, and "Preset Name — Modified" badge generation on manual slider overrides.
  - **Custom Preset Lifecycle**: Verifies saving, favoriting, duplicating, exporting to `.irispreset` JSON, and importing with schema validation and conflict resolution.
  - **Photo Auto Enhance Pipeline**: Validates bounded automatic color balance and tone correction on realistic photo inputs.

