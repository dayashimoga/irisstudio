# ADR-004: Memory Budgets & Cache Invalidation

## Status
Accepted (Phase 0)

## Context
High-resolution photo editing (48MP - 100MP camera RAW/JPEG) and 4K/60fps video timelines can easily exhaust physical RAM if frames or decoded bitmaps are retained indefinitely. Unbounded memory growth leads to OS out-of-memory (OOM) crashes, particularly on mobile devices where operating systems aggressively terminate processes using > 1.5 GB.

## Decision
1. **Explicit Memory Budgeting (`iris_cache`)**:
   - Iris Studio introduces a strict `CacheManager` with configurable memory limits:
     - Low-Memory / Mobile tier: 256 MB RAM cache.
     - Desktop standard tier: 512 MB - 1024 MB RAM cache.
     - Workstation tier: 2048 MB - 4096 MB RAM cache.
   - The cache stores decoded video frames, thumbnail bitmaps, and audio waveform data.
2. **LRU (Least Recently Used) Eviction Strategy**:
   - Every cached item tracks its byte size in memory.
   - When an insertion causes `total_bytes > max_budget_bytes`, the least recently used entries are evicted until `total_bytes <= max_budget_bytes * 0.9` (10% hysteresis buffer).
3. **Cache Invalidation on Mutation**:
   - Video Frames: Keyed by `(clip_id, frame_index, effect_hash)`. If a clip's color adjustments or transforms change, its `effect_hash` changes, rendering previous entries stale.
   - Waveforms: Keyed by `(audio_asset_id, sample_count, channel)`. Waveforms of raw source files are immutable and cached permanently on disk.
4. **Tiled Memory Bounds for Photos**:
   - Images exceeding 4096×4096 pixels are decomposed into 512×512 tiles for processing and caching, guaranteeing that interactive edits never allocate a single monolithic 100MP buffer in memory.

## Consequences
- Guaranteed upper bound on application memory usage.
- Elimination of memory leaks and OOM crashes during long editing sessions.
