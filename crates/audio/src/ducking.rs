use serde::{Deserialize, Serialize};

/// Audio ducker configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DuckingParameters {
    /// Detection threshold on dialogue track in dBFS (default: -28.0 dBFS).
    pub threshold_db: f32,
    /// Amount of background music attenuation in dB (default: -12.0 dB, range: -36.0 to 0.0 dB).
    pub duck_attenuation_db: f32,
    /// Attack duration in milliseconds for gain to drop (default: 20.0 ms).
    pub attack_ms: f32,
    /// Hold duration in milliseconds before release initiates (default: 350.0 ms).
    pub hold_ms: f32,
    /// Release duration in milliseconds for music volume to return (default: 500.0 ms).
    pub release_ms: f32,
}

impl Default for DuckingParameters {
    fn default() -> Self {
        Self {
            threshold_db: -28.0,
            duck_attenuation_db: -12.0,
            attack_ms: 20.0,
            hold_ms: 350.0,
            release_ms: 500.0,
        }
    }
}

/// Dynamic sidechain ducker that automatically attenuates background audio
/// when voiceover/dialogue energy is detected.
pub struct AudioDucker {
    params: DuckingParameters,
    sample_rate: u32,
    current_gain: f32,
    hold_remaining_samples: usize,
    envelope: f32,
}

impl AudioDucker {
    pub fn new(sample_rate: u32, params: DuckingParameters) -> Self {
        Self {
            params,
            sample_rate: sample_rate.max(8000),
            current_gain: 1.0,
            hold_remaining_samples: 0,
            envelope: 0.0,
        }
    }

    /// Processes stereo/mono background music samples in place based on voice dialogue stream.
    /// Both `dialogue_samples` and `music_samples` are expected to have the same frame count.
    pub fn process_interleaved(
        &mut self,
        dialogue_samples: &[f32],
        music_samples: &mut [f32],
        channels: usize,
    ) {
        let channels = channels.max(1);
        let frame_count = (music_samples.len() / channels).min(dialogue_samples.len() / channels);
        if frame_count == 0 {
            return;
        }

        let threshold_linear = 10.0f32.powf(self.params.threshold_db / 20.0);
        let min_gain = 10.0f32.powf(self.params.duck_attenuation_db.min(0.0) / 20.0);

        let attack_coeff =
            (-1.0 / (self.params.attack_ms.max(1.0) * 0.001 * self.sample_rate as f32)).exp();
        let release_coeff =
            (-1.0 / (self.params.release_ms.max(1.0) * 0.001 * self.sample_rate as f32)).exp();
        let hold_total_samples = (self.params.hold_ms * 0.001 * self.sample_rate as f32) as usize;

        // Fast RMS / peak envelope detector time constant (10ms)
        let env_coeff = (-1.0 / (0.010 * self.sample_rate as f32)).exp();

        for f in 0..frame_count {
            // Compute instantaneous voice amplitude across dialogue channels
            let mut voice_peak = 0.0f32;
            for ch in 0..channels {
                let v = dialogue_samples[f * channels + ch].abs();
                if v > voice_peak {
                    voice_peak = v;
                }
            }

            // Envelope follower
            if voice_peak > self.envelope {
                self.envelope = voice_peak;
            } else {
                self.envelope = self.envelope * env_coeff + voice_peak * (1.0 - env_coeff);
            }

            // Ducking state evaluation
            let target_gain = if self.envelope > threshold_linear {
                self.hold_remaining_samples = hold_total_samples;
                min_gain
            } else if self.hold_remaining_samples > 0 {
                self.hold_remaining_samples -= 1;
                min_gain
            } else {
                1.0
            };

            // Smooth gain ramping
            if target_gain < self.current_gain {
                self.current_gain =
                    self.current_gain * attack_coeff + target_gain * (1.0 - attack_coeff);
            } else {
                self.current_gain =
                    self.current_gain * release_coeff + target_gain * (1.0 - release_coeff);
            }

            // Apply gain to music channels
            for ch in 0..channels {
                music_samples[f * channels + ch] *= self.current_gain;
            }
        }
    }

    #[inline]
    pub fn current_gain(&self) -> f32 {
        self.current_gain
    }

    #[inline]
    pub fn current_gain_db(&self) -> f32 {
        if self.current_gain > 1e-5 {
            20.0 * self.current_gain.log10()
        } else {
            -100.0
        }
    }
}
