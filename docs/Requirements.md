# Iris Studio: System Requirements Specification

## 1. Executive Summary & Vision
**Iris Studio** is a professional, high-performance, cross-platform image and video editing workstation engineered for Windows, macOS, Linux, Android, and iOS/iPadOS. Built upon a unified Rust core engine (`iris_*` crates) with a responsive, modern Flutter interface, Iris Studio prioritizes:
- **Offline First**: Zero mandatory network connectivity, registration, cloud subscriptions, ads, or telemetry.
- **Nondestructive Pipeline**: Non-destructive editing graph where source media files are strictly read-only and all modifications are recorded as deterministic operations with unlimited undo/redo, auto-save checkpoints, and crash-journal recovery.
- **Low-Latency Preview**: Sub-50ms UI response time and bounded memory usage with intelligent LRU cache eviction and multi-resolution proxy workflows.
- **Cross-Platform Parity with Hardware Awareness**: Consistent project files and timeline behavior across mobile, tablet, and desktop, with explicit capability reporting rather than vague guarantees.

---

## 2. Platform Support & Minimum System Requirements

| Platform | Target OS Version | Minimum RAM | Recommended RAM | Graphics API Support | Primary Decoder HW |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Windows** | Windows 10 (1809+) / 11 | 4 GB | 16 GB+ | DirectX 11 / 12, Vulkan | Windows Media Foundation (DXVA2/D3D11VA) |
| **Linux** | Ubuntu 22.04+, Fedora 38+, Arch | 4 GB | 16 GB+ | Vulkan, OpenGL 4.3+ | VA-API / VDPAU |
| **macOS** | macOS 12 (Monterey) or later | 8 GB | 16 GB+ (Apple Silicon) | Metal 2.4+ | Apple VideoToolbox |
| **Android** | Android 9.0 (API 28)+ | 3 GB | 6 GB+ | Vulkan 1.1, OpenGL ES 3.2 | Android MediaCodec (NDK AImageReader/Surface) |
| **iOS / iPadOS**| iOS/iPadOS 15.0+ | 3 GB | 6 GB+ | Metal 2.4+ | Apple VideoToolbox (AVFoundation CoreVideo) |

---

## 3. Functional Requirements

### 3.1 Media Library & Ingestion
- **REQ-MED-01**: Ingestion of common photo formats: JPEG (JFIF/EXIF), PNG (8-bit/16-bit, RGBA), WebP (lossless & lossy with alpha).
- **REQ-MED-02**: Ingestion of standard video containers: MP4, MOV, WebM, MKV with support for H.264/AVC, H.265/HEVC, VP9, and AV1.
- **REQ-MED-03**: Audio stream demuxing: AAC, MP3, Opus, Linear PCM (16/24/32-bit float).
- **REQ-MED-04**: Handling of Variable Frame Rate (VFR) media through presentation timestamp (PTS) indexing and normalization to avoid audio drift.
- **REQ-MED-05**: EXIF/XMP/IPTC metadata extraction and orientation normalization (respecting EXIF Orientation tags 1-8 without lossy re-encoding).

### 3.2 Photo Workspace
- **REQ-PHT-01**: Canvas zoom (10% to 3200%), sub-pixel pan, smooth bilinear/bicubic rendering, and rotation.
- **REQ-PHT-02**: Nondestructive geometric transforms: freeform crop with aspect presets (1:1, 4:5, 16:9, 9:16, 3:2, Free), 90°/180° rotation, horizontal/vertical flip, and precision straightening (-45° to +45° with bicubic grid).
- **REQ-PHT-03**: Color & Exposure Adjustments:
  - Exposure (-5.0 to +5.0 EV)
  - Contrast (-100 to +100)
  - Temperature (2000K to 12000K) & Tint (-100 to +100)
  - Highlights & Shadows recovery
  - Saturation (-100 to +100) & Vibrance (-100 to +100 skin-tone preserving)
  - Parametric tone curves (RGB combined and per-channel R/G/B spline control)
  - Levels (Input black/white/gamma, Output black/white)
  - Sharpening (Unsharp Mask: radius, amount, threshold)
  - Gaussian Blur & Vignette (amount, midpoint, roundness, feather)
- **REQ-PHT-04**: Split Before/After view (interactive split bar slider and toggle).
- **REQ-PHT-05**: Real-time luminance and RGB histogram calculations.

### 3.3 Video Workspace
- **REQ-VID-01**: Multi-track non-linear editing timeline supporting arbitrary video, audio, title/text, and overlay tracks.
- **REQ-VID-02**: Frame-accurate trimming (In/Out points), splitting at playhead, duplicate, clip reordering, and ripple deletion.
- **REQ-VID-03**: Video transforms: Position (X/Y), Scale (uniform & non-uniform), Rotation (degrees), Opacity (0-100%), and Aspect Ratio canvas letterboxing/pillarboxing.
- **REQ-VID-04**: Transitions: Cross Dissolve, Wipe Left/Right/Up/Down, Slide, Fade to Black/White with configurable duration.
- **REQ-VID-05**: Dynamic Audio Processing:
  - Multichannel waveform generation (min/max peak envelopes at variable zoom resolutions).
  - Track & clip volume sliders (-60 dB to +12 dB) with linear and exponential gain curves.
  - Audio fade in / fade out handles.
  - Real-time stereo peak dBFS level meters.
- **REQ-VID-06**: Snapping to clip edges, markers, playhead, and second boundaries.
- **REQ-VID-07**: Timebases: Standard rational time bases (24/1, 25/1, 30/1, 50/1, 60/1, 30000/1001, 60000/1001 fps) with 100% frame-accurate integer tick math.

### 3.4 Project Management & Resilience
- **REQ-PRJ-01**: Pure JSON project schema (`.iris` / `.luma`) containing complete non-destructive edit graphs, track configurations, and asset references.
- **REQ-PRJ-02**: Relative asset path resolution with fallback relinking dialog if files are moved.
- **REQ-PRJ-03**: Infinite Undo/Redo stack with memory budget limits and action coalescing (e.g. continuous slider drags coalesce into single undo steps).
- **REQ-PRJ-04**: Atomic Auto-save running asynchronously at user-configurable intervals (default 60s) without UI stutter.
- **REQ-PRJ-05**: Write-Ahead-Log (WAL) Crash Recovery Journal: Every mutating command is recorded before execution. On unexpected termination, the journal prompts for 1-click recovery.

### 3.5 Export & Delivery
- **REQ-EXP-01**: Image Export: JPEG (configurable quality 1-100), PNG (compression level, 8-bit/16-bit), WebP (lossy/lossless), with metadata strip/preserve options and resolution resizing.
- **REQ-EXP-02**: Video Export: Presets (YouTube 4K/1080p, Social 9:16 Reels/Shorts, Archive Master), container selection (MP4, WebM), bitrate selection (CBR/VBR), audio bitrate (128-320 kbps), frame rate conversion.
- **REQ-EXP-03**: Export Queue: Background execution with progress percentage, rendered frames, average FPS, estimated remaining time, and cancellation support.
- **REQ-EXP-04**: Post-Export Verification: Re-opens generated media, verifies container headers, stream duration, dimensions, and decode integrity before notifying success.

### 3.6 Look Presets, Project Templates & Auto Adjust Engine
- **REQ-PST-01**: Strongly-typed Rust preset engine separating Look Presets, Project Templates, and Export Presets.
- **REQ-PST-02**: Built-in Video Looks: 15 tested, non-destructive configurations (Natural, Cinema Neutral, Cinema Warm, Cinema Cool, Film-Inspired, Documentary, Sports/Action, Travel, Landscape, Portrait/Interview, Product, Food, Night/Low-Light, Vintage, Black & White).
- **REQ-PST-03**: Built-in Photo Presets: 15 creative presets with full manual parameter overrides.
- **REQ-PST-04**: AI & Content-Aware Photo Auto Enhance: Statistical luminance, Gray-World balance, and dynamic highlight/shadow recovery.
- **REQ-PST-05**: Custom Presets & Portability: User look creation, duplicate, favorite, export/import `.irispreset`, offline schema sanitization.
- **REQ-PST-06**: Atomic Preset Application: Applied as 1 undoable action; manual slider adjustments flag "Preset Name — Modified"; intensity blending 0% to 100%.

---

## 4. Non-Functional Requirements

### 4.1 Performance Budgets
- **Cold App Startup**: < 1.5 seconds on desktop, < 2.0 seconds on mobile devices.
- **UI Input Response**: p95 latency < 50ms for slider manipulation, pan, and zoom operations.
- **Preview Playback**: Smooth real-time 1080p30 playback for baseline multi-track composition with basic color adjustments without frame drops.
- **Idle Memory Footprint**: Editor memory consumption ≤ 180MB RAM when idle with an empty project.
- **Memory Scaling**: RAM consumption bounded by configured cache budget (default 512MB RAM cache) regardless of input video duration.

### 4.2 Security & Sandbox
- **Untrusted File Parsing**: All image decoders and video container parsers treat input bytes as untrusted and enforce dimension caps (max 16384x16384 px) to prevent allocation bomb attacks (decompression bombs).
- **Local Privacy**: Zero outbound network requests. No telemetry, crash reporters, analytics pings, or font downloads at runtime.
- **Metadata Sanitization**: Export options explicitly allow stripping GPS coordinates, camera serial numbers, and author metadata for privacy.
