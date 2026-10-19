# Iris Studio: Testing Strategy

## 1. Quality Standards & Coverage Goals
- **Domain & Engine Code Coverage**: Minimum 90% branch/statement coverage for `iris_core`, `iris_timeline`, `iris_projects`, `iris_effects`, `iris_images`, and `iris_render`.
- **Zero Warnings Policy**: `cargo clippy --workspace --all-targets -- -D warnings` must produce zero warnings.
- **Flutter Analysis**: `flutter analyze` in `apps/flutter` must pass cleanly without info warnings.

---

## 2. Test Pyramid & Execution

### 2.1 Unit Tests (Rust)
- Pure deterministic logic:
  - Rational time math (`RationalTime::add`, `rescale`, `to_seconds`).
  - Nondestructive edit commands (`TrimCommand`, `SplitCommand`, `RippleDeleteCommand`).
  - Project JSON serialization and deserialization round-trip.
  - Pixel math accuracy: exposure formula, gamma conversion, Gaussian convolution matrix, unsharp masking.

### 2.2 Integration Tests (`/tests`)
- `tests/photo_pipeline_test.rs`: Loads test image, applies exposure, contrast, crop, and verifies output histogram and dimensions.
- `tests/timeline_composition_test.rs`: Assembles 3-track timeline with overlapping transition, evaluates frame at $t=1.5s$, verifies alpha blend.
- `tests/crash_recovery_test.rs`: Writes 10 commands to journal, simulates abrupt termination, re-opens project, verifies 100% state restoration.
- `tests/audio_waveform_test.rs`: Generates peak envelopes from synthetic sine/noise buffers, verifies min/max peak invariants.

### 2.3 Property-Based Testing
- Using `proptest` on timeline manipulation:
  - Any sequence of random splits, trims, and moves must maintain invariant that clips on a single track never overlap in timeline coordinates.

### 2.4 Fuzz Testing
- Fuzzing image decoders and project deserializers against malformed byte streams and decompression bombs.
