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
