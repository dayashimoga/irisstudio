# Iris Studio: Troubleshooting Guide

## 1. Common Issues & Solutions

### Issue: "Visual Studio not installed" warning in `flutter doctor`
- **Cause**: Flutter Windows desktop engine looks for MSVC tools. When developing with MinGW or testing on web/headless, desktop runners may require setting the C++ toolchain or building with GCC.
- **Solution**: Install Visual Studio Community with "Desktop development with C++" workload, or run in web/cross-target mode (`flutter run -d chrome`).

### Issue: Preview frame appears blank or black
- **Cause**: The input video codec or container has unusual pixel format (e.g. 10-bit HDR) that failed hardware decode initialization.
- **Solution**: Check the Diagnostics log in the UI. Iris Studio automatically falls back to software decode and applies tone mapping to 8-bit sRGB.

### Issue: Project crashed unexpectedly during editing
- **Cause**: Sudden OS reboot or out-of-memory event.
- **Solution**: Re-open Iris Studio. The Write-Ahead Log (`.iris-journal`) will automatically detect the uncommitted session and display a recovery banner: "Recover unsaved edits from previous session? [Restore] [Discard]".

### Issue: Audio waveform does not appear for newly imported file
- **Cause**: Waveform analysis is running asynchronously in the background.
- **Solution**: Iris Studio generates waveforms incrementally. Large multi-hour files display placeholder bounding guides while the background thread computes peak envelopes.

---

## 2. Incident Postmortem: P0 System-Stability Video-Import Desktop Freeze

### Problem Statement & Symptoms
When loading or scrubbing video media on Windows, the entire desktop became unresponsive, keyboard/mouse input lagged or froze, and dozens of invisible or transient popup console windows (`conhost.exe`) spawned in rapid succession.

### Root Cause Analysis (Proven with Code Locations)
1. **Windows Console Window Allocation Without `CREATE_NO_WINDOW`**:
   - In `crates/media/src/demux.rs`, `Command::new("ffmpeg")` and `Command::new("ffprobe")` were invoked without the Windows Win32 creation flag `CREATE_NO_WINDOW = 0x08000000`.
   - On Windows, launching a console subsystem binary from a GUI application without this flag triggers the OS kernel and Desktop Window Manager (`dwm.exe`) to allocate a console host process (`conhost.exe`).
2. **Per-Frame Probe and Process Spawning Loop**:
   - `RealMediaDecoder::decode_video_frame()` re-invoked `Self::probe_file()` unconditionally on **every single frame decode**.
   - During scrubbing or timeline playback at 15–30 FPS, this triggered `ffprobe -version`, `ffprobe`, `ffmpeg -version`, and `ffmpeg` synchronously on every render step (~60 process spawns per second), creating a severe context-switch storm in DWM and `csrss.exe`.
3. **Unconsumed Pipes and Unbounded Execution**:
   - External commands lacked redirected stdin (`Stdio::null()`), redirected stdout/stderr (`Stdio::piped()`), and execution timeouts. A stalled subprocess blocked Flutter's engine loop indefinitely.
4. **Unmanaged Scripting & Duplicate Dialogs**:
   - `MediaPickerDialog` invoked an unmanaged `Process.run('powershell', ...)` script and lacked a reentrancy lock, allowing rapid clicks to stack duplicate modal pickers.

### Architecture Repairs & Mitigations Implemented
1. **Zero Console Window Guarantee**:
   - All subprocess commands across Rust (`crates/media/src/demux.rs`, `hardware.rs`) now route through `create_safe_command()`, explicitly applying `.creation_flags(0x08000000)` (`CREATE_NO_WINDOW`), `stdin(Stdio::null())`, and bounded `stdout`/`stderr` pipes.
2. **Pure-Rust Demuxing Prioritization**:
   - `IsoBmffParser::probe(&path)` operates purely in-memory (<0.5ms) without disk thrashing, network calls, or spawning external helper processes.
3. **Process Concurrency & Caching Guards**:
   - `FFMPEG_AVAILABLE` and `FFPROBE_AVAILABLE` check results are memoized for the process lifetime using `OnceLock<bool>`.
   - Single-flight atomic execution guard `FFMPEG_ACTIVE` limits active decoding processes to at most 1.
   - In-memory 16-slot LRU `FRAME_CACHE` serves repeated timestamps instantaneously with 0 subprocesses.
4. **Hard Timeout & Process Tree Termination**:
   - `run_command_with_timeout(cmd, timeout_ms)` polls child status and terminates hanging helpers with `.kill()` and `.wait()`, preventing orphan worker leaks.
5. **UI State Machine & Backpressure**:
   - `IrisEngineController` implements an explicit `MediaImportStatus` state machine: `idle` → `picking` → `probing` → `initializing` → `ready`, with `failed` and `cancelled` terminal handling.
   - `cancelImport()` provides an interactive "Cancel Loading" button in the viewport that aborts in-flight work and releases resources immediately.
   - Playhead frame decoding is rate-limited via `_isDecodingFrame` and `_pendingDecodeSeconds`, dropping stale intermediate scrubbing positions.
6. **Dialog Single-Instance Guard**:
   - `MediaPickerDialog` enforces `_isPickerActive` mutex lock, returning `null` on duplicate invocations and eliminating unmanaged PowerShell execution.

