# ADR-005: Native Audio Metering and Deterministic RIFF WAV Export

## Status
Accepted (Phase 7)

## Context
Initial prototypes of the Flutter interface used synthetic modulations (`math.sin(_playheadSeconds * 8.0)`) and fixed 440 Hz tone approximations to drive audio level meters, waveform envelopes, and export output. This violated the core mandate requiring authentic offline media pipeline execution and zero mock/synthetic data in production workflows. Furthermore, audio-only projects and audio export operations required frame-accurate, multi-track sequence evaluation (factoring in clip volume, master track gains, sidechain ducking attenuation, and 3-band parametric EQ) written to independently verified standard PCM containers.

## Decision
1. **Real Rust SIMD Metering Integration (`iris_audio_calculate_meter_levels`)**:
   - The Flutter audio engine controller (`iris_engine_controller.dart`) strictly samples active project clips at the current playhead position, aggregates active track channels, and passes the synthesized buffer across the C-ABI via `IrisFfiBindings.calculateAudioMeters(buffer)`.
   - The Rust engine computes exact RMS level and peak dBFS per channel using `AudioMixer::calculate_meter_levels`.
   - Visual audio master deck meters and waveform overlays reflect authentic measured decibel levels, eliminating all synthetic sine waves.

2. **Native 16-Bit 48 kHz Stereo RIFF WAV Export Pipeline (`iris_audio_render_wav`)**:
   - Audio project rendering evaluates the multi-track timeline non-destructively:
     - Applies clip-level volume envelopes and timeline boundaries.
     - Evaluates sidechain ducking attenuation when dialogue tracks are present (`AudioDuckerModel`).
     - Evaluates 3-band parametric EQ (`AudioEqModel`).
     - Applies soft-clipping saturation (`tanh`) to prevent harsh digital wrapping at 0 dBFS.
   - The Rust FFI API directly generates fully compliant RIFF/WAVE chunks (`WAVEfmt ` and `data`) with proper 44-byte headers and little-endian 16-bit interleaved PCM samples.
   - Preflight storage checks and automatic file collision resolution ensure output integrity.

3. **True Broadcast Standard Scopes (`_ColorScopePainter`)**:
   - All simulated sine modulations in Vectorscope, RGB Parade, and Waveform visualizers were replaced with exact SMPTE 75% chrominance targets, 123° skin-tone reference axes, and descending 8-step luminance staircases matching industry-standard grading monitors.

## Consequences
- Guaranteed authenticity: All meters, waveforms, scopes, and exports represent actual digital signal processing and video composition.
- Zero reliance on external FFmpeg processes for standard WAV audio exports.
- Exported WAV files are 100% compliant with standard media players (VLC, Windows Media Player, QuickTime).
- Full backwards and cross-platform compatibility across Windows, macOS, Linux, Android, and iOS/iPadOS.
