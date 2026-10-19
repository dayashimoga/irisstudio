# ADR-003: Codec Licensing & Hardware Acceleration

## Status
Accepted (Phase 0)

## Context
Video decoders and encoders (H.264/AVC, H.265/HEVC, AAC) are covered by complex patent pools (MPEG LA, HEVC Advance, Via Licensing). Distributing unencumbered software binaries containing patent-encumbered codec implementations can trigger significant royalty liabilities and legal constraints. Furthermore, hardware decoders are substantially more power-efficient.

## Decision
1. **Prioritize Native Hardware Decoders/Encoders**:
   - **Windows**: Use Windows Media Foundation (WMF) APIs (`MFCreateSourceReaderFromURL`, `IMFTransform`). The OS handles the underlying hardware decoders (NVIDIA NVDEC, Intel QuickSync, AMD VCN) and associated licensing.
   - **Apple (macOS / iOS)**: Use Apple VideoToolbox (`VTDecompressionSession`, `VTCompressionSession`) and AVFoundation.
   - **Android**: Use Android NDK `AMediaCodec` / `AMediaExtractor`.
   - **Linux**: Target VA-API (`libva`) for hardware acceleration with royalty-free royalty-clean software fallbacks.
2. **Royalty-Free Open Codecs in Base Distribution**:
   - WebM container, VP8/VP9, and AV1 (via `dav1d` for decoding, `rav1e` for encoding).
   - Audio: Opus, Vorbis, FLAC, and Uncompressed Linear PCM.
3. **Software Fallback Policy**:
   - For royalty-encumbered formats (H.264/AAC), rely on OS-provided system decoders.
   - Clearly report format support status in the UI rather than failing silently.

## Consequences
- Clean licensing posture for commercial and open-source redistribution.
- Lower power consumption and thermal footprint across all target platforms.
