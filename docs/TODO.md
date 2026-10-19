# Iris Studio: Master Task List & Roadmap

## Phase 0: Feasibility & Foundation (Current Phase)
- [x] Create comprehensive architectural documentation, ADRs, matrices, and governance.
- [x] Establish Cargo workspace and modular crate boundaries.
- [ ] Implement foundational core crates:
  - [ ] `iris_core`: Rational time, color space, geometry, errors.
  - [ ] `iris_media`: Demuxer, frame types, hardware decode abstraction.
  - [ ] `iris_images`: Image transforms, codecs, crop/rotate/straighten, tiled processing.
  - [ ] `iris_timeline`: Tracks, clips, transitions, keyframes, nondestructive edits.
  - [ ] `iris_effects`: Exposure, contrast, white balance, curves, levels, blur, sharpening, vignette.
  - [ ] `iris_render`: Compositing engine, layer blending, double-buffering, backpressure.
  - [ ] `iris_audio`: Waveforms, mixing, volume envelopes, stereo metering.
  - [ ] `iris_projects`: JSON schema, undo/redo, atomic autosave, WAL crash journal.
  - [ ] `iris_cache`: Bounded LRU cache, memory budgeting, proxy management.
  - [ ] `iris_export`: Range export, batch photo processing, validation pass.
  - [ ] `iris_ffi`: Typed C-ABI interface for Flutter.
- [ ] Implement Flutter Adaptive Application (`apps/flutter`):
  - [ ] Modern UI design system (dark aesthetic, clean typography, responsive layout).
  - [ ] Home / Projects workspace.
  - [ ] Photo workspace with interactive canvas, before/after slider, and parametric controls.
  - [ ] Video workspace with multitrack timeline, playhead, snapping, and waveform display.
  - [ ] Export workspace with presets, codec configuration, and progress display.
  - [ ] FFI Bridge linking Dart to Rust engine with fallback modes.
- [ ] Create unit, integration, and property tests.
- [ ] Setup CI workflow (`.github/workflows/ci.yml`).
- [ ] Validate Phase 0 verification gates.

## Phase 1: Photo MVP
- [ ] Camera RAW workflow integration.
- [ ] Lens distortion correction.
- [ ] Red-eye and clone/heal brush tools.
- [ ] Batch image export with naming rules.

## Phase 2: Video MVP
- [ ] Extended transition presets (Iris, Zoom, Slide, Dip to Color).
- [ ] Audio EQ and multi-band compression.
- [ ] Title card generator with typography styles.
- [ ] Stream copy optimization for unedited timeline cuts.
