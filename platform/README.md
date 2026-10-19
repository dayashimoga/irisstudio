# Platform Native Graphics Surface & Codec Integrations

This directory isolates the native platform hardware interfaces for Iris Studio:

| Platform | Primary API | Graphics Surface | Native Hardware Decoders |
| :--- | :--- | :--- | :--- |
| **Windows** | Direct3D 11 / DXGI | `flutter::TextureRegistrar` shared D3D11 texture | Windows Media Foundation (`IMFSourceReader`) |
| **macOS / iOS** | Metal | `CVMetalTextureCache` / `CVPixelBuffer` | Apple VideoToolbox (`VTDecompressionSession`) |
| **Android** | Vulkan / NDK | `AHardwareBuffer` / `SurfaceTexture` | Android MediaCodec (`AMediaCodec`) |
| **Linux** | Vulkan / EGL | DMA-BUF shared memory texture | VA-API (`libva`) / VDPAU |

All platform integrations are decoupled from domain crates via the clean abstractions in `iris_media` and `iris_render`.
