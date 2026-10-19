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
