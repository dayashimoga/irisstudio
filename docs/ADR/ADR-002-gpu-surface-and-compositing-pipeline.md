# ADR-002: GPU Surface & Compositing Pipeline

## Status
Accepted (Phase 0)

## Context
Video playback and image manipulation require rendering frames at up to 4K resolution at 60fps. Copying 3840×2160 RGBA frames (33.2 MB per frame) between CPU and GPU memory at 60 fps consumes ~2.0 GB/s of system memory bandwidth, causing memory bus contention, battery drain, and thermal throttling on mobile devices.

## Decision
1. **Compositing Engine (`iris_render`)**:
   - Implements a declarative render graph where nodes represent source decoders, spatial transforms (matrix affine transforms), pixel shaders/effects, and layer blend modes.
   - Operates in linear RGB color space to ensure mathematically correct optical blending and anti-aliasing.
2. **Preview Presentation**:
   - **Primary Path (Hardware Interop)**: On Windows, use Direct3D 11 shared textures via Flutter Windows texture registrar (`flutter::TextureRegistrar`). On macOS/iOS, use Metal `CVMetalTextureCache` / `CVPixelBuffer`. On Android, use `AHardwareBuffer` / `SurfaceTexture`.
   - **Deterministic Fallback (Software Buffers)**: When native GPU texture registration is unavailable or fails initial handshake, Iris Studio falls back to a pinned double-buffered BGRA/RGBA pixel memory mapped directly into Flutter's `ui.decodeImageFromPixels` or custom pixel buffer. This ensures the app operates reliably even on virtual machines, low-end devices, or unsupported graphics drivers.
3. **Backpressure & Frame Dropping**:
   - The render scheduler tracks the presentation display VSync.
   - If the render thread falls behind the timeline playhead, intermediate frames are skipped in favor of the latest timestamp, preventing unbounded queue buildup.

## Consequences
- Guaranteed zero lockups and smooth UI independently of media frame rate.
- Verified fallback path ensures Iris Studio launches and functions on any system.
