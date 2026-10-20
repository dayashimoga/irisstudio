use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MeterLevels {
    pub left_dbfs: f32,
    pub right_dbfs: f32,
    pub peak_left: f32,
    pub peak_right: f32,
}

pub struct AudioMixer;

impl AudioMixer {
    /// Mixes multiple audio buffers into a single interleaved master stereo buffer.
    pub fn mix_stereo_tracks(tracks: &[(&[f32], f32)]) -> Vec<f32> {
        if tracks.is_empty() {
            return Vec::new();
        }

        let max_len = tracks.iter().map(|(buf, _)| buf.len()).max().unwrap_or(0);
        let mut master = vec![0.0f32; max_len];

        for (buf, gain) in tracks {
            for (i, sample) in buf.iter().enumerate() {
                master[i] += sample * gain;
            }
        }

        // Soft-clip limiter to prevent digital distortion wrapping
        for sample in &mut master {
            *sample = sample.tanh();
        }

        master
    }

    /// Calculates stereo RMS and peak levels in dBFS.
    pub fn calculate_meter_levels(stereo_samples: &[f32]) -> MeterLevels {
        if stereo_samples.is_empty() {
            return MeterLevels {
                left_dbfs: -100.0,
                right_dbfs: -100.0,
                peak_left: 0.0,
                peak_right: 0.0,
            };
        }

        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        let mut sum_sq_l = 0.0f32;
        let mut sum_sq_r = 0.0f32;
        let pairs = stereo_samples.len() / 2;

        for &[l_raw, r_raw] in stereo_samples.as_chunks::<2>().0 {
            let l = l_raw.abs();
            let r = r_raw.abs();

            if l > peak_l {
                peak_l = l;
            }
            if r > peak_r {
                peak_r = r;
            }

            sum_sq_l += l * l;
            sum_sq_r += r * r;
        }

        let rms_l = (sum_sq_l / pairs.max(1) as f32).sqrt();
        let rms_r = (sum_sq_r / pairs.max(1) as f32).sqrt();

        let left_dbfs = if rms_l > 1e-5 {
            20.0 * rms_l.log10()
        } else {
            -100.0
        };
        let right_dbfs = if rms_r > 1e-5 {
            20.0 * rms_r.log10()
        } else {
            -100.0
        };

        MeterLevels {
            left_dbfs: left_dbfs.clamp(-100.0, 0.0),
            right_dbfs: right_dbfs.clamp(-100.0, 0.0),
            peak_left: peak_l.clamp(0.0, 1.0),
            peak_right: peak_r.clamp(0.0, 1.0),
        }
    }
}
