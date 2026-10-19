# Iris Studio: User Manual & Operator Guide

Welcome to **Iris Studio**, a professional-grade, 100% offline cross-platform video and image workstation built with high-performance **Rust Stable** for core processing and an adaptive modern **Flutter** interface.

---

## 1. Product Principles & Offline Guarantee

Iris Studio operates under strict product principles:
- **100% Offline by Design**: All image transformations, video decoding, color grading, audio dynamics, optical flow interpolation, and exporting run locally on your device hardware.
- **Zero Registration, Cloud, or Subscriptions**: No sign-in required, no telemetry, no tracking, and no external web dependencies.
- **Non-Destructive Editing Model**: Source media files are never overwritten. Every edit, cut, grade, and effect is recorded in portable, human-readable JSON project journals (`.irisproj`) with atomic save and crash-recovery journals.
- **Bounded Resource Budgets**: Strictly enforced frame-cache budgets prevent unbounded memory growth; whole-video-in-RAM decoding is forbidden.

---

## 2. Workspace Navigation

Iris Studio provides 4 unified workspaces accessible from the top navigation bar:

| Workspace | Purpose |
| :--- | :--- |
| **Home / Projects** | Project management, asset library, template creation, and recent projects |
| **Photo Workspace** | Raw image editing, multi-layer composition, retouching, and batch export |
| **Video Workspace** | Multitrack timeline editing, audio dynamics, color grading, optical flow, and multicam |
| **Export Workspace** | Multi-preset render queue, format transcoding, stream-copy export, and delivery |

---

## 3. Video Editing Workspace

### 3.1 Multitrack Timeline & Playhead Navigation
- **Playhead**: Drag the cyan playhead ruler across the timeline to scrub frames with sub-frame timecode display (`HH:MM:SS:FF`).
- **Transport Controls**:
  - `Play / Pause`: Spacebar or center transport button.
  - `Step Frame`: Jump backward/forward by exactly 1 frame.
  - `Split Clip (S)`: Split the selected or active clip at the current playhead position without quality loss.
  - `Ripple Delete (Shift + Del)`: Remove selected clip and automatically shift succeeding clips forward to eliminate gaps.
  - `Timeline Snapping`: Magnetically snaps playhead and clip boundaries to adjacent edits and transition points.
  - `Timeline Zoom`: Zoom horizontal scale from 0.2x to 5.0x for frame-accurate precision.

### 3.2 Stream Copy Optimizer (25x - 30x Export Acceleration)
Whenever clips retain default transform, normal speed, and no active transitions, Iris Studio automatically bypasses expensive software re-encoding:
- Unedited cuts are directly remuxed at raw SSD/NVMe disk I/O speeds (up to 30x faster than full rendering).
- The Master Inspector features a live **Stream Copy Eligibility Indicator** showing the exact percentage of timeline duration qualifying for zero-loss passthrough.

### 3.3 Audio Suite & Dynamics Processing
- **3-Band Parametric EQ**: Robert Bristow-Johnson biquad IIR filter with Low Shelf (120 Hz), Mid Peaking (1 kHz), and High Shelf (8 kHz) spanning $-18\text{ dB}$ to $+18\text{ dB}$.
- **Audio Dynamics Compressor**: Configurable threshold ($-30$ to $0\text{ dBFS}$), compression ratio (1:1 to 20:1 limiter), envelope follower, and makeup gain ($0$ to $+24\text{ dB}$).
- **Dialogue Sidechain Ducking**: Automatically attenuates background music and ambient scores by up to $-30\text{ dB}$ when vocal speech is detected, preventing audio clashing without pumping.
- **3D Spatial Audio & Binaural HRTF**: First-order Ambisonics ($W, X, Y, Z$) with spherical head diffraction (Woodworth ITD + acoustic shadow ILD) for immersive headphone spatialization.
- **WSOLA Pitch-Preserving Time-Stretch**: Smooth playback speed adjustment from 0.25x to 4.0x while preserving natural voice and musical pitch.

### 3.4 Color Grading, 3D LUTs & HDR Tone Mapping
- **Diagnostic Color Scopes**: Real-time picture-in-picture (PIP) Luma Waveform (256-level IRE density), polar Vectorscope with 123° skin-tone axis, and 3-channel RGB Parade.
- **3D LUT Engine**: Imports industry-standard `.cube` lattice files with high-speed 4-vertex tetrahedral interpolation and built-in cinematic presets (*Teal & Orange, Warm Vintage, Bleach Bypass, Faded Film, Monochrome*).
- **Parametric RGB Curves**: Monotonic cubic Hermite spline interpolation preventing unnatural overshoot ringing when manipulating tonal contrast.
- **HDR Tone Mapping**: SMPTE ST 2084 PQ and ARIB STD-B67 HLG transfer functions with ACES filmic S-curve and Extended Reinhard tone mappers.

### 3.5 Multi-Camera Switching & Quad-View Monitor
- **Synchronized Camera Groups**: Ingest up to 4 synchronized camera angles (`CAM 1` - `CAM 4`) with individual millisecond timecode sync offsets.
- **Live Angle Switching**: Click angle chips in the Master Inspector or tap directly on the live video canvas to cut angles in real time during playback.
- **Interactive 2x2 Quad-View Monitor**: Displays all 4 cameras simultaneously with broadcast `ON AIR` pills and active camera highlight borders.
- **Non-Destructive Flattening**: Export multi-cam cut sequences into standard timeline clips at any time.

### 3.6 Optical Flow Frame Interpolation (AI Slow-Motion)
- **Bidirectional Motion Fields**: Block-matching motion estimation with sub-pixel parabolic peak refinement.
- **Forward-Backward Occlusion Guard**: Prevents edge disocclusion tearing and ghosting artifacts.
- **Slow-Motion Multipliers**: Synthesizes 2x, 4x, and 8x slow-motion sequences with natural motion continuity.

### 3.7 Smart Rotoscoping & Edge Masking
- **Contrast-Aware Edge Detection**: 3x3 Sobel gradient fields establish boundary barriers for complex subject silhouettes.
- **Interactive Geodesic Matting**: Propagates color affinity from foreground and background seed points.
- **Bilateral Feathering**: Produces clean, anti-aliased alpha mattes without pixelation or staircasing.

---

## 4. Photo Editing Workspace

### 4.1 Non-Destructive Multi-Layer Composition
- Reorder layers, toggle visibility, and adjust layer opacity (0% - 100%).
- 7 Industry-standard blend modes: *Normal, Multiply, Screen, Overlay, Darken, Lighten, Difference*.

### 4.2 Retouching Suite
- **Inpainting Healing Brush**: Replaces blemishes and unwanted objects using gradient-consistent neighborhood interpolation.
- **Clone Stamp**: Duplicates pixel textures from sampled reference coordinates.
- **Red-Eye Removal**: Perceptually detects unnatural pupil redness and restores natural dark pigmentation.
- **Bilateral Denoise**: Eliminates high-ISO sensor noise while preserving sharp contrast edges.

### 4.3 Batch Photo Processing Pipeline
- Process hundreds of images concurrently with Rayon multi-core execution.
- Resize, watermarking, lens correction, and format conversion (JPEG, PNG, WebP) with real-time job queues and progress indicators.

---

## 5. Keyboard Shortcuts Reference

| Shortcut | Action |
| :--- | :--- |
| **Space** | Play / Pause playback |
| **S** | Split selected clip at playhead position |
| **Shift + Delete** | Ripple delete selected clip and close gap |
| **Left Arrow** | Step backward 1 frame |
| **Right Arrow** | Step forward 1 frame |
| **Ctrl + Z** | Undo last action |
| **Ctrl + Y / Ctrl + Shift + Z** | Redo last undone action |
| **Ctrl + S** | Save project atomically to disk |
| **1 / 2 / 3 / 4** | Switch to Camera Angle 1, 2, 3, or 4 |

---

## 6. Technical Support & Verification

To verify your installation and ensure all native modules pass quality gates, run the included test suite from PowerShell:
```powershell
powershell.exe -ExecutionPolicy Bypass -File scripts/run_tests.ps1
```
All 48 Rust unit and integration tests, Clippy zero-warnings gate, code formatting, and Flutter analyzer checks will execute and report complete verification.
