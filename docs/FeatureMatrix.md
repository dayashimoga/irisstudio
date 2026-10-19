# Iris Studio: Feature Support Matrix by Platform

| Capability / Feature | Windows | Linux | macOS | Android | iOS / iPadOS | Web Fallback |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Photo Editing Core** | | | | | | |
| JPEG / PNG / WebP Import & Export | Full | Full | Full | Full | Full | Full |
| Nondestructive Geometric Transforms | Full | Full | Full | Full | Full | Full |
| Parametric Adjustments (Exposure, WB, Curves) | Full | Full | Full | Full | Full | Full |
| Interactive Split Before/After View | Full | Full | Full | Full | Full | Full |
| Real-time Histograms (RGB/Luma) | Full | Full | Full | Full | Full | Full |
| Tiled Processing for Mega-Pixel Images | Full | Full | Full | Full | Full | Full |
| Batch Photo Processing & Renaming | Full | Full | Full | Full | Full | Limited |
| **Video Editing Core** | | | | | | |
| Multi-Track Timeline (Video/Audio/Titles) | Full | Full | Full | Full | Full | Full |
| Frame-Accurate Trim, Split, Ripple Delete | Full | Full | Full | Full | Full | Full |
| Video Transforms (Scale, Position, Rotate) | Full | Full | Full | Full | Full | Full |
| Transitions (Dissolve, Wipe, Slide) | Full | Full | Full | Full | Full | Full |
| Multichannel Audio Waveforms | Full | Full | Full | Full | Full | Full |
| Volume Envelopes & Fades | Full | Full | Full | Full | Full | Full |
| Snapping & Fractional Timebase Math | Full | Full | Full | Full | Full | Full |
| Video Export (MP4 / WebM) | Full | Full | Full | Full | Full | Limited |
| **Hardware Acceleration** | | | | | | |
| HW Video Decoding | MediaFoundation | VA-API | VideoToolbox | MediaCodec | VideoToolbox | Browser Video |
| HW Video Encoding | MediaFoundation | VA-API | VideoToolbox | MediaCodec | VideoToolbox | WebCodecs |
| GPU Surface Presentation | D3D11 / Shared | Vulkan | Metal | AHardwareBuffer| Metal | WebGL2 / Canvas |
| **Resilience & Storage** | | | | | | |
| Pure JSON Project Files (`.iris`) | Full | Full | Full | Full | Full | Full |
| Infinite Undo / Redo with Memory Limit | Full | Full | Full | Full | Full | Full |
| Atomic Auto-Save | Full | Full | Full | Full | Full | LocalStorage |
| WAL Crash Recovery Journal | Full | Full | Full | Full | Full | IndexedDB |
| Offline-Only Operation (Zero Telemetry) | 100% | 100% | 100% | 100% | 100% | 100% |
