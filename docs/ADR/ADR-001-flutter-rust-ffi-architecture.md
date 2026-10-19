# ADR-001: Flutter & Rust FFI Architecture

## Status
Accepted (Phase 0)

## Context
Iris Studio requires high-performance, frame-accurate media manipulation, SIMD-accelerated image algorithms, multitrack timeline evaluation, and low-latency audio mixing. Concurrently, the user interface must be cross-platform (Windows, Linux, macOS, Android, iOS), highly responsive, accessible, and visually stunning.

Evaluating architectures:
1. **Pure Rust UI (e.g. Iced, egui, Slint, Tauri)**: Egui and Iced are promising but lack mature accessibility trees, text input handling for complex scripts, mobile ecosystem plugins, and touch gesture refinement across iOS/Android. Tauri relies on web views which introduce DOM overhead and latency for frame presentation.
2. **Pure Flutter**: Dart lacks zero-cost SIMD abstractions, fine-grained thread scheduling, predictable real-time memory management (due to GC pauses), and native hardware decoding control.
3. **Rust Core Engine + Flutter Adaptive UI via Typed FFI**: Combines Rust's memory safety, predictable performance, and concurrency with Flutter's widget ecosystem, multi-device adaptivity, and 120Hz smooth animations.

## Decision
We adopt **Rust Core Engine + Flutter UI** with a narrow, typed C-ABI FFI boundary:
- The Rust engine is compiled into a dynamic library (`iris_core.dll`, `libiris_core.so`, `libiris_core.dylib`).
- Dart communicates with Rust via `dart:ffi` using C-compatible structs and opaque pointers.
- Large media buffers (video frames, decoded pixel maps, audio PCM) are pinned in native memory and referenced by pointer/length rather than copied across Dart message channels.
- State mutation requests (e.g., `trim_clip`, `set_exposure`) are synchronous or asynchronous C functions returning structured result codes and JSON payloads.
- High-frequency preview frame delivery utilizes shared memory double-buffering.

## Consequences
- **Pros**: Maximum computational throughput for image/video/audio in Rust; 100% native Flutter UI feel on mobile and desktop; clean separation of concerns.
- **Cons**: Requires maintaining FFI bindings and handling cross-language memory lifetimes carefully to prevent leaks or use-after-free bugs.
