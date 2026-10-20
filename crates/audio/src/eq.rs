use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EqualizerBand {
    pub frequency_hz: f32,
    pub gain_db: f32,  // -18.0 to +18.0 dB
    pub q_factor: f32, // typically 0.5 to 2.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreeBandEq {
    pub low: EqualizerBand,  // Low-shelf (default 120 Hz)
    pub mid: EqualizerBand,  // Peaking bell (default 1000 Hz)
    pub high: EqualizerBand, // High-shelf (default 8000 Hz)
    pub sample_rate: f32,
}

impl Default for ThreeBandEq {
    fn default() -> Self {
        Self {
            low: EqualizerBand {
                frequency_hz: 120.0,
                gain_db: 0.0,
                q_factor: 0.707,
            },
            mid: EqualizerBand {
                frequency_hz: 1000.0,
                gain_db: 0.0,
                q_factor: 1.0,
            },
            high: EqualizerBand {
                frequency_hz: 8000.0,
                gain_db: 0.0,
                q_factor: 0.707,
            },
            sample_rate: 48000.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

#[derive(Debug, Clone, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    #[inline]
    fn process(&mut self, input: f32, coeff: &BiquadCoefficients) -> f32 {
        let output = coeff.b0 * input + coeff.b1 * self.x1 + coeff.b2 * self.x2
            - coeff.a1 * self.y1
            - coeff.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

impl ThreeBandEq {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    fn calculate_low_shelf(band: &EqualizerBand, sr: f32) -> BiquadCoefficients {
        let a = 10.0f32.powf(band.gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * band.frequency_hz / sr;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / 2.0
            * ((a + 1.0 / a) * (1.0 / band.q_factor - 1.0) + 2.0)
                .max(0.0)
                .sqrt();
        let beta = 2.0 * a.sqrt() * alpha;

        let a0 = (a + 1.0) + (a - 1.0) * cs + beta;
        let inv_a0 = 1.0 / a0.max(1e-6);

        BiquadCoefficients {
            b0: (a * ((a + 1.0) - (a - 1.0) * cs + beta)) * inv_a0,
            b1: (2.0 * a * ((a - 1.0) - (a + 1.0) * cs)) * inv_a0,
            b2: (a * ((a + 1.0) - (a - 1.0) * cs - beta)) * inv_a0,
            a1: (-2.0 * ((a - 1.0) + (a + 1.0) * cs)) * inv_a0,
            a2: ((a + 1.0) + (a - 1.0) * cs - beta) * inv_a0,
        }
    }

    fn calculate_peaking(band: &EqualizerBand, sr: f32) -> BiquadCoefficients {
        let a = 10.0f32.powf(band.gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * band.frequency_hz / sr;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * band.q_factor.max(0.1));

        let a0 = 1.0 + alpha / a;
        let inv_a0 = 1.0 / a0.max(1e-6);

        BiquadCoefficients {
            b0: (1.0 + alpha * a) * inv_a0,
            b1: (-2.0 * cs) * inv_a0,
            b2: (1.0 - alpha * a) * inv_a0,
            a1: (-2.0 * cs) * inv_a0,
            a2: (1.0 - alpha / a) * inv_a0,
        }
    }

    fn calculate_high_shelf(band: &EqualizerBand, sr: f32) -> BiquadCoefficients {
        let a = 10.0f32.powf(band.gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * band.frequency_hz / sr;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / 2.0
            * ((a + 1.0 / a) * (1.0 / band.q_factor - 1.0) + 2.0)
                .max(0.0)
                .sqrt();
        let beta = 2.0 * a.sqrt() * alpha;

        let a0 = (a + 1.0) - (a - 1.0) * cs + beta;
        let inv_a0 = 1.0 / a0.max(1e-6);

        BiquadCoefficients {
            b0: (a * ((a + 1.0) + (a - 1.0) * cs + beta)) * inv_a0,
            b1: (-2.0 * a * ((a - 1.0) + (a + 1.0) * cs)) * inv_a0,
            b2: (a * ((a + 1.0) + (a - 1.0) * cs - beta)) * inv_a0,
            a1: (2.0 * ((a - 1.0) - (a + 1.0) * cs)) * inv_a0,
            a2: ((a + 1.0) - (a - 1.0) * cs - beta) * inv_a0,
        }
    }

    /// Processes an interleaved stereo buffer in-place through the 3-band EQ.
    pub fn process_stereo(&self, samples: &mut [f32]) {
        if self.low.gain_db == 0.0 && self.mid.gain_db == 0.0 && self.high.gain_db == 0.0 {
            return; // Fast path bypass
        }

        let low_coeff = Self::calculate_low_shelf(&self.low, self.sample_rate);
        let mid_coeff = Self::calculate_peaking(&self.mid, self.sample_rate);
        let high_coeff = Self::calculate_high_shelf(&self.high, self.sample_rate);

        let mut low_state_l = BiquadState::default();
        let mut low_state_r = BiquadState::default();
        let mut mid_state_l = BiquadState::default();
        let mut mid_state_r = BiquadState::default();
        let mut high_state_l = BiquadState::default();
        let mut high_state_r = BiquadState::default();

        for chunk in samples.as_chunks_mut::<2>().0 {
            let mut l = chunk[0];
            let mut r = chunk[1];

            // 1. Low-shelf
            l = low_state_l.process(l, &low_coeff);
            r = low_state_r.process(r, &low_coeff);

            // 2. Peaking Mid
            l = mid_state_l.process(l, &mid_coeff);
            r = mid_state_r.process(r, &mid_coeff);

            // 3. High-shelf
            l = high_state_l.process(l, &high_coeff);
            r = high_state_r.process(r, &high_coeff);

            chunk[0] = l.clamp(-1.0, 1.0);
            chunk[1] = r.clamp(-1.0, 1.0);
        }
    }
}

/// Dynamic range compressor and peak limiter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioCompressor {
    pub threshold_dbfs: f32, // -30.0 to 0.0 dBFS
    pub ratio: f32,          // 1.0 (no compression) to 20.0 (limiting)
    pub attack_ms: f32,      // 0.1 to 100.0 ms
    pub release_ms: f32,     // 10.0 to 1000.0 ms
    pub makeup_gain_db: f32, // 0.0 to 24.0 dB
}

impl Default for AudioCompressor {
    fn default() -> Self {
        Self {
            threshold_dbfs: -18.0,
            ratio: 4.0,
            attack_ms: 5.0,
            release_ms: 100.0,
            makeup_gain_db: 3.0,
        }
    }
}

impl AudioCompressor {
    pub fn process_stereo(&self, samples: &mut [f32], sample_rate: f32) {
        let makeup_linear = 10.0f32.powf(self.makeup_gain_db / 20.0);
        let attack_coeff = (-1.0 / (self.attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * sample_rate)).exp();

        let mut envelope = 0.0f32;

        for chunk in samples.as_chunks_mut::<2>().0 {
            let peak = chunk[0].abs().max(chunk[1].abs());

            if peak > envelope {
                envelope = attack_coeff * envelope + (1.0 - attack_coeff) * peak;
            } else {
                envelope = release_coeff * envelope + (1.0 - release_coeff) * peak;
            }

            let env_dbfs = if envelope > 1e-5 {
                20.0 * envelope.log10()
            } else {
                -100.0
            };

            let gain_db = if env_dbfs > self.threshold_dbfs {
                let over_db = env_dbfs - self.threshold_dbfs;
                -over_db * (1.0 - 1.0 / self.ratio.max(1.0))
            } else {
                0.0
            };

            let gain_linear = 10.0f32.powf(gain_db / 20.0) * makeup_linear;

            chunk[0] = (chunk[0] * gain_linear).tanh();
            chunk[1] = (chunk[1] * gain_linear).tanh();
        }
    }
}
