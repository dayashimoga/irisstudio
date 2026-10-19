# Iris Studio: Codebase Understanding & Architecture Walkthrough

## 1. Domain Entities & Core Invariants

### 1.1 Time & Frame Indexing (`iris_core`)
Media timelines must never rely on floating-point seconds due to precision drift over hours of playback.
- `RationalTime`: Represents an exact time as `numerator / denominator` (e.g. `1001 / 30000` for 29.97 fps NTSC).
- `TimeSpan`: Composed of a `start: RationalTime` and `duration: RationalTime`. All range tests, overlaps, and trimming operations use exact integer rational math.

### 1.2 Timeline Composition Model (`iris_timeline`)
- A `Timeline` contains multiple `Track` objects ordered by z-index.
- A `Track` contains a list of non-overlapping `Clip` instances.
- A `Clip` references a `SourceMedia` (video, image, audio, or solid/title), with an `in_point` (trim in), `out_point` (trim out), and `timeline_start`.
- `Transition` (e.g. Cross Dissolve, Wipe) bridges two contiguous clips on the same track with an overlap duration.

### 1.3 Compositing Pipeline (`iris_render`)
When the UI requests a preview frame at time `t`:
1. The `Compositor` determines all active clips intersecting `t`.
2. For each active video clip, the source frame is retrieved from the `CacheManager` (or requested from `MediaDecoder`).
3. The clip's `Transform` matrix (translation, scale, rotation) and opacity are computed.
4. Any active `EffectChain` is applied (exposure, contrast, curves).
5. Layers are blended onto a destination canvas in linear color space according to their blend mode (Normal, Multiply, Screen, etc.).
6. The resulting buffer is delivered to the Flutter presentation surface.

### 1.4 Write-Ahead Log (WAL) Crash Recovery (`iris_projects`)
- Any command modifying the timeline (such as `SplitClip`, `ChangeAdjustment`) is written to `.iris-journal` before mutating memory.
- An atomic counter tracks sequence numbers.
- Upon opening a project, the engine checks for the presence of `.iris-journal`. If found, it replays the journal commands automatically.
