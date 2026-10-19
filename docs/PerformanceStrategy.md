# Iris Studio: Performance Strategy & Benchmarks

## 1. Concrete Performance Baselines & Targets

| Metric | Target Baseline | Measured Target | Test Condition |
| :--- | :---: | :---: | :--- |
| **Cold App Startup** | < 2.0 s | < 1.2 s | Windows / Linux / macOS desktop (Core i5 / Ryzen 5 / Apple M1) |
| **Idle Memory (RAM)** | ≤ 200 MB | ~110 MB | Empty project, idle editor after startup |
| **UI Input Latency** | p95 < 50 ms | < 16 ms | Continuous slider adjustments (60fps UI loop) |
| **1080p30 Playback** | 30.0 fps | 30.0 fps (0 dropped) | 3-track composite with exposure + vignette adjustment |
| **4K Scrub Latency** | < 100 ms | < 65 ms | Seek to random timeline position using preview proxy |
| **Peak Memory Bounding** | ≤ Configured Budget | 512 MB (Default) | Long video (2+ hours 4K timeline) with active editing |

---

## 2. Engineering Optimizations

### 2.1 SIMD & Parallelism
- Rust pixel pipelines leverage LLVM auto-vectorization and explicitly aligned memory chunks (e.g. processing 4 or 8 pixels per AVX2/NEON iteration).
- Multi-core parallel image processing powered by `rayon` slices for independent scanlines and tiles.

### 2.2 Tiled Mega-Pixel Operations
- High-resolution camera photos (e.g. 50MP - 100MP) are partitioned into 512×512 tiles.
- Interactive adjustments in the viewport only compute visible tiles for the current zoom level, eliminating multi-gigabyte memory spikes and lag.

### 2.3 Non-Blocking UI Architecture
- All media decoding, audio mixing, project file writes, and export rendering run on background thread pools.
- Preview frame delivery uses double-buffered memory slots with lockless atomic state swaps.
