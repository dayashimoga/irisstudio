# Iris Studio: Developer Guide

## 1. Project Organization

The workspace is organized into modular Rust crates and an adaptive Flutter application:

```
/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── iris_core/              # Common primitives, timebases, color formats
│   ├── iris_media/             # Media decoding, demuxing, container parsing
│   ├── iris_images/            # High-performance image transforms & codecs
│   ├── iris_timeline/          # Nondestructive tracks, clips, transitions
│   ├── iris_effects/           # Color adjustments, curves, blur, vignette
│   ├── iris_render/            # Compositor, render graph, double-buffering
│   ├── iris_audio/             # Audio waveforms, mixing, volume envelopes
│   ├── iris_projects/          # Portable JSON project format, undo/redo, WAL
│   ├── iris_cache/             # Bounded multi-tier LRU frame & disk cache
│   ├── iris_export/            # Multi-threaded export scheduler & validation
│   └── iris_ffi/               # C-ABI export layer for Flutter integration
├── apps/
│   └── flutter/                # Cross-platform adaptive UI
├── platform/                   # Platform-specific native glue
├── tests/                      # End-to-end and integration tests
├── benchmarks/                 # Criterion microbenchmarks
├── scripts/                    # Automation and CI verification scripts
└── docs/                       # Architectural specifications & guides
```

---

## 2. Coding Standards & Lints

### Rust
- Strict formatting: `cargo fmt --all -- --check`
- Zero clippy warnings allowed: `cargo clippy --workspace --all-targets -- -D warnings`
- Memory Safety: `#![forbid(unsafe_code)]` on all domain crates (`iris_core`, `iris_timeline`, `iris_projects`, `iris_effects`). Unsafe code is isolated strictly to FFI boundary pointers in `iris_ffi` and platform graphics handles, thoroughly documented with `// SAFETY:` invariants.
- No Panics in Production Paths: Use `Result<T, IrisError>` everywhere.

### Flutter / Dart
- Strict analysis options: `flutter analyze` must pass with zero issues.
- State Management: Reactive, immutable state models with uncoupled UI widgets.
- Accessibility: All buttons and interactive sliders must provide semantic labels and tooltips.

---

## 3. Adding a New Effect or Adjustment

1. Define the parameter struct in `crates/iris_effects/src/adjustment.rs`.
2. Implement the pixel processing function operating on `&mut [Rgba8]` or `&mut [RgbaF32]`.
3. Add the adjustment to the `EffectChain` pipeline in `crates/iris_render`.
4. Expose the adjustment parameter in `crates/iris_ffi/src/api.rs`.
5. Connect the adjustment slider in `apps/flutter/lib/workspaces/photo/`.
6. Add unit test asserting output pixel values against mathematical reference.
