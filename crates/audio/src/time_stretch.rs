use std::f32::consts::PI;

/// High-performance WSOLA (Waveform Similarity Overlap-Add) time-stretch processor.
/// Enables playback speed adjustment (e.g. 0.25x - 4.0x) while preserving original pitch.
pub struct WsolaTimeStretch {
    sample_rate: u32,
    channels: usize,
    window_size: usize,
    synthesis_hop: usize,
    search_range: usize,
    hann_window: Vec<f32>,
}

impl WsolaTimeStretch {
    pub fn new(sample_rate: u32, channels: usize) -> Self {
        let channels = channels.max(1);
        // Default 1024 sample window (~21ms at 48kHz), 512 hop, 256 search window
        let window_size = 1024;
        let synthesis_hop = 512;
        let search_range = 256;

        let mut hann_window = vec![0.0f32; window_size];
        for (n, w) in hann_window.iter_mut().enumerate() {
            *w = 0.5 * (1.0 - (2.0 * PI * n as f32 / (window_size - 1) as f32).cos());
        }

        Self {
            sample_rate,
            channels,
            window_size,
            synthesis_hop,
            search_range,
            hann_window,
        }
    }

    /// Stretches or compresses the interleaved audio buffer by `speed_ratio` without altering pitch.
    /// - `speed_ratio > 1.0`: Faster playback (shorter duration).
    /// - `speed_ratio < 1.0`: Slower playback (longer duration).
    /// - `speed_ratio == 1.0`: Pass-through.
    pub fn process(&self, input: &[f32], speed_ratio: f32) -> Vec<f32> {
        let speed_ratio = speed_ratio.clamp(0.25, 4.0);
        if (speed_ratio - 1.0).abs() < 1e-4 {
            return input.to_vec();
        }

        let num_frames = input.len() / self.channels;
        if num_frames < self.window_size + self.search_range * 2 {
            // Buffer too short for full WSOLA framing; return cloned buffer
            return input.to_vec();
        }

        let output_frames = ((num_frames as f32 / speed_ratio) as usize).max(self.window_size);
        let mut output = vec![0.0f32; output_frames * self.channels];
        let mut window_weights = vec![0.0f32; output_frames];

        let mut synth_frame = 0;
        let mut prev_chosen_analysis = 0;

        while synth_frame + self.window_size <= output_frames {
            // Target analysis frame in input
            let nominal_analysis = (synth_frame as f32 * speed_ratio).round() as isize;

            // Search for optimal offset within [-search_range, +search_range] to maximize similarity
            let best_offset = if synth_frame == 0 {
                0
            } else {
                let template_start = prev_chosen_analysis + self.synthesis_hop;
                self.find_best_offset(input, num_frames, template_start, nominal_analysis)
            };

            let chosen_analysis = (nominal_analysis + best_offset)
                .clamp(0, (num_frames.saturating_sub(self.window_size)) as isize)
                as usize;

            // Overlap and add windowed frame to output
            for w in 0..self.window_size {
                let out_idx = synth_frame + w;
                if out_idx >= output_frames {
                    break;
                }
                let in_idx = chosen_analysis + w;
                let win = self.hann_window[w];

                for ch in 0..self.channels {
                    output[out_idx * self.channels + ch] +=
                        input[in_idx * self.channels + ch] * win;
                }
                window_weights[out_idx] += win;
            }

            prev_chosen_analysis = chosen_analysis;
            synth_frame += self.synthesis_hop;
        }

        // Normalize output by accumulated window weights
        for f in 0..output_frames {
            let weight = window_weights[f];
            if weight > 1e-5 {
                let inv = 1.0 / weight;
                for ch in 0..self.channels {
                    output[f * self.channels + ch] *= inv;
                }
            }
        }

        output
    }

    /// Finds offset delta within search range that maximizes cross-correlation with natural continuation template.
    fn find_best_offset(
        &self,
        input: &[f32],
        total_frames: usize,
        template_start: usize,
        nominal_center: isize,
    ) -> isize {
        if template_start + self.window_size > total_frames {
            return 0;
        }

        let max_search = self.search_range as isize;
        let mut best_offset = 0;
        let mut max_corr = f32::NEG_INFINITY;

        // Downsampled correlation for fast cross-correlation search (step by 2 frames)
        for delta in (-max_search..=max_search).step_by(2) {
            let candidate_start = nominal_center + delta;
            if candidate_start < 0 || (candidate_start as usize) + self.window_size > total_frames {
                continue;
            }

            let c_start = candidate_start as usize;
            let mut corr = 0.0f32;

            // Compute dot product across primary/mono channel with stride 4 for SIMD friendliness
            let stride = 4;
            for i in (0..self.window_size).step_by(stride) {
                let t_sample = input[(template_start + i) * self.channels];
                let c_sample = input[(c_start + i) * self.channels];
                corr += t_sample * c_sample;
            }

            if corr > max_corr {
                max_corr = corr;
                best_offset = delta;
            }
        }

        best_offset
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
