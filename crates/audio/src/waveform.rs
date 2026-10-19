use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaveformPeak {
    pub min: f32, // negative peak, e.g. -0.8
    pub max: f32, // positive peak, e.g. +0.85
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveformEnvelope {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples_per_pixel: usize,
    pub peaks: Vec<WaveformPeak>,
}

pub fn generate_waveform_peaks(
    samples: &[f32],
    channels: u16,
    samples_per_pixel: usize,
) -> WaveformEnvelope {
    assert!(channels > 0);
    let spp = samples_per_pixel.max(1);
    let total_frames = samples.len() / channels as usize;
    let num_peaks = total_frames.div_ceil(spp);

    let mut peaks = Vec::with_capacity(num_peaks);

    for bucket in 0..num_peaks {
        let start_frame = bucket * spp;
        let end_frame = ((bucket + 1) * spp).min(total_frames);

        let mut min_val = 0.0f32;
        let mut max_val = 0.0f32;

        for frame in start_frame..end_frame {
            for ch in 0..channels as usize {
                let sample = samples[frame * channels as usize + ch];
                if sample < min_val {
                    min_val = sample;
                }
                if sample > max_val {
                    max_val = sample;
                }
            }
        }

        peaks.push(WaveformPeak {
            min: min_val.clamp(-1.0, 1.0),
            max: max_val.clamp(-1.0, 1.0),
        });
    }

    WaveformEnvelope {
        sample_rate: 48000,
        channels,
        samples_per_pixel: spp,
        peaks,
    }
}
