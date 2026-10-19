# Iris Studio: Sequential Implementation Plan & Verification Gates

## Phase Overview

The project is structured into nine sequential phases. Each phase has concrete deliverables and objective verification gates that must pass before advancing.

```
Phase 0: Feasibility & Foundation ──> Phase 1: Photo MVP ──> Phase 2: Video MVP ──>
Phase 3: Everyday Editing ──> Phase 4: Performance & Large Projects ──>
Phase 5: Full Platform Rollout ──> Phase 6: Advanced Editing ──>
Phase 7: Optional Local AI ──> Phase 8: Production Hardening
```

---

## Phase 0: Feasibility & Foundation (Current Phase)
- **Objectives**:
  - Establish complete documentation, ADRs, matrices, and governance.
  - Set up Cargo workspace with modular crates (`iris_core`, `iris_media`, `iris_images`, `iris_timeline`, `iris_effects`, `iris_render`, `iris_audio`, `iris_projects`, `iris_cache`, `iris_export`, `iris_ffi`).
  - Prove Flutter-Rust typed FFI presentation, preview frame generation, audio waveform generation, non-destructive state manipulation, and project persistence.
  - Implement baseline test suites, CI workflows, and benchmark harness.
- **Verification Gates**:
  1. `cargo test --workspace` passes with zero failures.
  2. `cargo clippy --workspace -- -D warnings` passes with zero warnings.
  3. Flutter analysis passes cleanly.
  4. Working Flutter desktop application launches and renders real-time photo adjustments and video playback preview.

---

## Phase 1: Photo MVP
- **Objectives**:
  - Full photo canvas with pan, pinch zoom, rotate, crop with aspect presets.
  - Parametric adjustments: Exposure, Contrast, White Balance, Highlights/Shadows, Saturation, Vibrance, Levels, Curves, Sharpening, Blur, Vignette.
  - Split Before/After comparison.
  - Nondestructive `.iris` project saving, atomic autosave, and crash recovery.
  - Export to JPEG, PNG, WebP with custom dimensions and quality.
- **Verification Gates**:
  1. Photo import/export round-trip preserves color accuracy and metadata.
  2. Continuous adjustment slider moves maintain >60fps UI response.
  3. Crash simulation recovery succeeds with 100% edit restoration.

---

## Phase 2: Video MVP
- **Objectives**:
  - Multi-track timeline (Video, Audio, Titles) with snap-to-grid.
  - Frame-accurate trim in/out, split at playhead, ripple delete, clip reorder.
  - Video transforms (scale, position, rotation, opacity).
  - Cross dissolve and wipe transitions.
  - Multichannel waveform generation and volume envelopes.
  - Real-time preview player with transport controls and timecode display.
  - Export to MP4/WebM with post-export verification check.
- **Verification Gates**:
  1. No audio/video synchronization drift (<1 frame) across 10-minute playback test.
  2. Playback of 1080p30 timeline sustains 30 fps without frame drops.
  3. Export file plays correctly in standard system media players.

---

## Phase 3: Everyday Editing
- **Objectives**:
  - Unlimited multi-track composition with track solo/mute.
  - Keyframe editor with linear and ease-in/out interpolation for transforms and volume.
  - Speed changes (0.25x to 8x) and reverse playback.
  - Title and caption generator with text styling and burn-in export.
  - Batch photo conversion tool with naming rules and collisions handling.
  - Command palette / quick search for tools, effects, and shortcuts.
- **Verification Gates**:
  1. Keyframe interpolation math matches reference curves within 0.01% tolerance.
  2. Batch export of 50 images completes with zero memory leaks.

---

## Phase 4: Performance & Large Projects
- **Objectives**:
  - Multi-resolution proxy generation (1/2, 1/4, 1/8 resolution).
  - Tiled image pipeline for 100MP+ images.
  - Bounded memory cache with LRU eviction and memory budget compliance.
  - Hardware accelerated decoding (Media Foundation, VideoToolbox, MediaCodec).
  - Export queue manager with background rendering.
- **Verification Gates**:
  1. Total idle RAM remains < 180MB; peak editing RAM stays within configured budget.
  2. 4K timeline scrub latency p95 < 80ms using proxies.

---

## Phase 5: Full Platform Rollout
- **Objectives**:
  - Native Windows, Linux, macOS, Android, and iOS builds.
  - Mobile adaptive UI layouts (phone bottom sheets, tablet split screen, touch targets).
  - File picker integration, scoped storage compliance, and permission handling.
- **Verification Gates**:
  1. Responsive UI renders seamlessly across phone (360x800), tablet (1024x768), and 4K desktop.
  2. Clean build and packaging on all platforms.

---

## Phase 6: Advanced Editing
- **Objectives**:
  - Motion tracking and video stabilization.
  - Chroma keying (green/blue screen) with despill and edge feathering.
  - Curves and color wheels (Lift/Gamma/Gain).
  - Nested sequences and adjustment layers.
- **Verification Gates**:
  1. Chroma key achieves clean matte on standard test footage with <15ms render cost per frame.

---

## Phase 7: Optional Local AI
- **Objectives**:
  - Offline transcription and automatic subtitle generation.
  - Local background removal and portrait segmentation.
  - Separately downloadable model packages with explicit user consent.
- **Verification Gates**:
  1. 100% offline execution; zero network requests during inference.
  2. CPU fallback executes when NPU/GPU is unavailable.

---

## Phase 8: Production Hardening
- **Objectives**:
  - Comprehensive fuzzing of media parsers and project deserializers.
  - Automated long-session stability testing (8+ hours editing).
  - Accessibility audit (screen readers, high contrast, reduced motion).
  - Signed release packages, SBOMs, and changelog finalization.
- **Verification Gates**:
  1. Zero memory leaks or crashes over 100,000 fuzzing iterations.
  2. Release binaries signed and verified.
