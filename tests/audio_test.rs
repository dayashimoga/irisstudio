use iris_audio::{generate_waveform_peaks, AudioMixer};
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
