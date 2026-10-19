# Iris Studio: Codec & Container Compatibility Matrix

## 1. Video Containers & Demuxing

| Container | File Extensions | Demuxer Support | Stream Copy Support | Platform Availability | Notes |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **MP4 / MOV** | `.mp4`, `.m4v`, `.mov` | Full | Supported | All Platforms | ISO/IEC 14496-12 standard atom parsing |
| **WebM** | `.webm` | Full | Supported | All Platforms | Matroska subset with VP8/VP9/AV1/Opus |
| **Matroska** | `.mkv` | Full | Supported | Desktop Platforms | Extended MKV container parsing |

---

## 2. Video Codecs & Color Formats

| Codec | Decode Method | Encode Method | Max Resolution | Pixel Formats | HDR Support |
| :--- | :--- | :--- | :---: | :--- | :---: |
| **H.264 / AVC** | OS HW Decoder (WMF, VT, MediaCodec) | OS HW Encoder | 4096×2160 (Level 5.2) | YUV420p, NV12 | SDR |
| **H.265 / HEVC** | OS HW Decoder (WMF, VT, MediaCodec) | OS HW Encoder | 8192×4320 (Main 10) | NV12, P010 | HDR10 / HLG (with tone mapping) |
| **VP9** | Native libvpx / Dav1d / OS HW | WebM Muxer | 7680×4320 (Profile 0/2) | YUV420p, YUV420p10 | HDR10 |
| **AV1** | Native `dav1d` / OS HW | `rav1e` / OS HW | 7680×4320 | YUV420p, YUV420p10 | HDR10 |

---

## 3. Audio Formats & Codecs

| Format | Decode Support | Encode Support | Sample Rates | Bit Depths | Channel Layouts |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **AAC-LC** | Native / OS | Native / OS | 44.1 kHz, 48 kHz | 16-bit, Float | Mono, Stereo, 5.1 Surround |
| **Opus** | Full (Native) | Full (Native) | 48 kHz (Native) | Float32 | Mono, Stereo |
| **MP3** | Full (Native) | Full (Native) | 44.1 kHz, 48 kHz | 16-bit, Float | Mono, Stereo |
| **WAV / PCM**| Full (Native) | Full (Native) | 8 kHz to 192 kHz | 16, 24, 32-bit float | All layouts up to 7.1 |
| **FLAC** | Full (Native) | Full (Native) | Up to 192 kHz | 16, 24-bit | Mono, Stereo |

---

## 4. Image Formats

| Format | Decode Support | Encode Support | Max Dimension | Color Depths | Transparency |
| :--- | :---: | :---: | :---: | :--- | :---: |
| **JPEG** | Full (Native) | Full (Native) | 16384×16384 px | 8-bit sRGB | No |
| **PNG** | Full (Native) | Full (Native) | 16384×16384 px | 8-bit, 16-bit RGBA | Full Alpha Channel |
| **WebP** | Full (Native) | Full (Native) | 16384×16384 px | 8-bit RGBA (lossless/lossy)| Full Alpha Channel |
| **BMP / TIFF**| Full (Native) | Export optional | 16384×16384 px | 8-bit, 16-bit RGB/RGBA | Supported |

---

## 5. Color Spaces & Precision
- **Internal Pipeline**: Working color space is 32-bit floating point Linear sRGB / Rec.709 with premultiplied alpha for all compositing and effects.
- **Color Transform**: Non-linear transfer functions (sRGB EOTF, Rec.709 gamma, BT.2020) are linearized prior to blending and requantized on export to prevent gamma distortion and dark fringes.
