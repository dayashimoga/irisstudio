# Iris Studio: Risk Register

| Risk ID | Category | Description | Impact | Likelihood | Mitigation Strategy |
| :--- | :--- | :--- | :---: | :---: | :--- |
| **RSK-01** | Legal / IP | MPEG LA / HEVC patent royalty claims for distributed binaries | Critical | Low | Prioritize OS-provided hardware decoders (WMF, VideoToolbox, MediaCodec) where the OS vendor holds licenses. Keep bundled software codecs restricted to royalty-free formats (AV1, VP9, Opus, Vorbis). |
| **RSK-02** | Stability | High-resolution image/video allocations causing OS Out-Of-Memory termination | High | Medium | Implement hard memory budgets in `iris_cache`, LRU frame eviction, and tiled 512×512 processing for mega-pixel images. |
| **RSK-03** | Interop | Flutter-Rust FFI memory pointer misuse or lifecycle mismatches | High | Low | Enforce opaque C handles with Rust-managed lifecycle ownership. Encapsulate memory transfers into pinned buffers with explicit free functions. |
| **RSK-04** | Performance | Audio/Video desynchronization over long playback sessions | High | Medium | Use exact integer `RationalTime` arithmetic instead of floating-point timestamps; lock audio clock as master reference clock. |
| **RSK-05** | Security | Decompression bombs or malicious media files crafted to crash the host | Critical | Low | Strictly validate dimensions and frame buffer sizes before memory allocation; cap dimensions to 16,384×16,384 px. |
