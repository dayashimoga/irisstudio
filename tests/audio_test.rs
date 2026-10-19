use iris_audio::{
    generate_waveform_peaks, AudioCompressor, AudioMixer, EqualizerBand, ThreeBandEq,
};
use iris_media::AudioFrame;

#[test]
fn test_audio_waveform_peaks() {
    // Generate 48kHz test tone (1 second = 48000 samples)
    let frame = AudioFrame::create_test_tone(48000, 2, 440.0, 1.0, 0.75);
    assert_eq!(frame.samples.len(), 96000); // 2 channels

    let spp = 480; // 100 peaks per second
    let envelope = generate_waveform_peaks(&frame.samples, frame.channels, spp);

    assert_eq!(envelope.peaks.len(), 100);
    for peak in &envelope.peaks {
        assert!(peak.max > 0.0);
        assert!(peak.min < 0.0);
        assert!(peak.max <= 0.76);
        assert!(peak.min >= -0.76);
    }
}

#[test]
fn test_audio_stereo_mixer_and_metering() {
    let tone1 = AudioFrame::create_test_tone(48000, 2, 440.0, 0.1, 0.5);
    let tone2 = AudioFrame::create_test_tone(48000, 2, 880.0, 0.1, 0.3);

    let mixed = AudioMixer::mix_stereo_tracks(&[(&tone1.samples, 1.0), (&tone2.samples, 0.8)]);

    assert_eq!(mixed.len(), tone1.samples.len());

    let levels = AudioMixer::calculate_meter_levels(&mixed);
    assert!(levels.peak_left > 0.0);
    assert!(levels.peak_right > 0.0);
    assert!(levels.left_dbfs <= 0.0);
    assert!(levels.right_dbfs <= 0.0);
}

#[test]
fn test_three_band_eq() {
    let frame = AudioFrame::create_test_tone(48000, 2, 100.0, 0.05, 0.5);
    let mut samples = frame.samples.clone();

    // Default EQ: 0 dB across all bands => bypass / no alteration
    let default_eq = ThreeBandEq::default();
    let original = samples.clone();
    default_eq.process_stereo(&mut samples);
    assert_eq!(samples, original);

    // Boost low shelf by +6 dB
    let mut eq = ThreeBandEq::new(48000.0);
    eq.low = EqualizerBand {
        frequency_hz: 120.0,
        gain_db: 6.0,
        q_factor: 0.707,
    };
    eq.process_stereo(&mut samples);

    let levels_before = AudioMixer::calculate_meter_levels(&original);
    let levels_after = AudioMixer::calculate_meter_levels(&samples);
    // After +6 dB low shelf boost on 100 Hz tone, RMS and peak should be higher
    assert!(levels_after.peak_left > levels_before.peak_left);
    assert!(levels_after.left_dbfs > levels_before.left_dbfs);
}

#[test]
fn test_audio_compressor() {
    // High amplitude burst tone
    let frame = AudioFrame::create_test_tone(48000, 2, 1000.0, 0.05, 0.95);
    let mut samples = frame.samples.clone();

    let comp = AudioCompressor {
        threshold_dbfs: -12.0,
        ratio: 4.0,
        attack_ms: 1.0,
        release_ms: 50.0,
        makeup_gain_db: 0.0,
    };

    comp.process_stereo(&mut samples, 48000.0);

    let orig_levels = AudioMixer::calculate_meter_levels(&frame.samples);
    let comp_levels = AudioMixer::calculate_meter_levels(&samples);

    // Peak levels should be compressed down
    assert!(comp_levels.peak_left < orig_levels.peak_left);
}

#[test]
fn test_audio_time_stretch_wsola() {
    let sample_rate = 48000;
    // 0.5 seconds at 440 Hz
    let frame = AudioFrame::create_test_tone(sample_rate, 2, 440.0, 0.5, 0.6);
    let wsola = iris_audio::WsolaTimeStretch::new(sample_rate, 2);

    // 1.0x pass-through
    let normal = wsola.process(&frame.samples, 1.0);
    assert_eq!(normal.len(), frame.samples.len());

    // 0.5x slow motion (duration should double)
    let slow = wsola.process(&frame.samples, 0.5);
    let expected_slow_len = frame.samples.len() * 2;
    // Check length is within 10% of 2x
    let diff_slow = (slow.len() as isize - expected_slow_len as isize).abs();
    assert!(diff_slow < (expected_slow_len as isize / 10));

    // 2.0x fast forward (duration should halve)
    let fast = wsola.process(&frame.samples, 2.0);
    let expected_fast_len = frame.samples.len() / 2;
    let diff_fast = (fast.len() as isize - expected_fast_len as isize).abs();
    assert!(diff_fast < (expected_fast_len as isize / 10));

    // Verify all samples are finite and non-empty
    for s in &slow {
        assert!(s.is_finite());
    }
    for s in &fast {
        assert!(s.is_finite());
    }
}

#[test]
fn test_audio_ducking_sidechain() {
    let sample_rate = 48000;
    let duration_sec: f64 = 0.8;
    let total_frames = (sample_rate as f64 * duration_sec) as usize;

    // Music track: continuous 440 Hz tone at 0.5 amplitude
    let music = AudioFrame::create_test_tone(sample_rate, 2, 440.0, duration_sec, 0.5);
    let mut music_samples = music.samples.clone();

    // Dialogue track: Silence for 0.0 - 0.2s, then loud voice (1000 Hz, 0.8 amp) for 0.2 - 0.5s, then silence
    let mut dialogue_samples = vec![0.0f32; total_frames * 2];
    let speech_start = (sample_rate as f32 * 0.2) as usize;
    let speech_end = (sample_rate as f32 * 0.5) as usize;
    for f in speech_start..speech_end {
        let t = f as f32 / sample_rate as f32;
        let s = (2.0 * std::f32::consts::PI * 1000.0 * t).sin() * 0.8;
        dialogue_samples[f * 2] = s;
        dialogue_samples[f * 2 + 1] = s;
    }

    let mut ducker = iris_audio::AudioDucker::new(
        sample_rate,
        iris_audio::DuckingParameters {
            threshold_db: -20.0,
            duck_attenuation_db: -12.0, // ~0.251 linear
            attack_ms: 10.0,
            hold_ms: 100.0,
            release_ms: 150.0,
        },
    );

    ducker.process_interleaved(&dialogue_samples, &mut music_samples, 2);

    // Check amplitude before speech (e.g. at 0.1s)
    let pre_idx = (sample_rate as f32 * 0.1) as usize * 2;
    let pre_speech_amp = music_samples[pre_idx].abs();

    // Check amplitude during speech (e.g. at 0.35s)
    let during_idx = (sample_rate as f32 * 0.35) as usize * 2;
    let during_speech_amp = music_samples[during_idx].abs();

    // Due to -12 dB ducking, the music peak during dialogue should be significantly attenuated
    assert!(
        during_speech_amp < pre_speech_amp * 0.7,
        "Music should be ducked during active speech (during: {}, pre: {})",
        during_speech_amp,
        pre_speech_amp
    );
}
