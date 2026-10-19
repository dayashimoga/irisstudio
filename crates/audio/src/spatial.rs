use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// A single sample frame of First-Order Ambisonics (FOA) in B-format.
/// Follows standard Furse-Malham (FuMa) / ACN conventions.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct BFormatFrame {
    /// Omnidirectional pressure component (0th order spherical harmonic)
    pub w: f32,
    /// Front-Back figure-of-eight directional component (1st order cosine azimuth)
    pub x: f32,
    /// Left-Right figure-of-eight directional component (1st order sine azimuth)
    pub y: f32,
    /// Up-Down figure-of-eight directional component (1st order sine elevation)
    pub z: f32,
}

impl BFormatFrame {
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// Encodes a mono point source into an Ambisonics B-format frame at given spherical coordinates.
    /// - `azimuth_rad`: Horizontal angle in radians (-PI to +PI, 0 = straight ahead, +PI/2 = left, -PI/2 = right).
    /// - `elevation_rad`: Vertical angle in radians (-PI/2 to +PI/2, 0 = horizontal plane, +PI/2 = directly above).
    pub fn from_point_source(sample: f32, azimuth_rad: f32, elevation_rad: f32) -> Self {
        let cos_elev = elevation_rad.cos();
        let sin_elev = elevation_rad.sin();
        let cos_azim = azimuth_rad.cos();
        let sin_azim = azimuth_rad.sin();

        // 1/sqrt(2) FuMa normalization for W
        let w = sample * std::f32::consts::FRAC_1_SQRT_2;
        let x = sample * cos_elev * cos_azim;
        let y = sample * cos_elev * sin_azim;
        let z = sample * sin_elev;

        Self { w, x, y, z }
    }
}

/// 3D Spatial Audio Virtual Panner using spherical head-diffraction physics (Woodworth model).
/// Converts 3D positional audio into realistic binaural stereo with interaural time (ITD)
/// and interaural level (ILD) head shadowing.
pub struct BinauralPanner {
    sample_rate: f32,
    head_radius: f32,    // in meters (approx 0.0875m human average)
    speed_of_sound: f32, // approx 343.0 m/s
    // Ring delay buffers for left and right ears to model ITD
    left_delay_buf: Vec<f32>,
    right_delay_buf: Vec<f32>,
    write_idx: usize,
    // 1-pole head shadow filter states
    left_shadow_state: f32,
    right_shadow_state: f32,
}

impl BinauralPanner {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000) as f32;
        // Max delay buffer size for ~10ms delay (far exceeds max head radius ITD of ~0.7ms)
        let buf_size = ((sr * 0.01) as usize).max(128);

        Self {
            sample_rate: sr,
            head_radius: 0.0875,
            speed_of_sound: 343.0,
            left_delay_buf: vec![0.0; buf_size],
            right_delay_buf: vec![0.0; buf_size],
            write_idx: 0,
            left_shadow_state: 0.0,
            right_shadow_state: 0.0,
        }
    }

    /// Processes a buffer of mono audio samples into an interleaved stereo binaural buffer
    /// at the given 3D position (azimuth in degrees, elevation in degrees, distance in meters).
    pub fn process_mono_to_binaural(
        &mut self,
        mono_in: &[f32],
        stereo_out: &mut [f32],
        azimuth_deg: f32,
        elevation_deg: f32,
        distance_meters: f32,
    ) {
        let azim_rad = azimuth_deg.clamp(-180.0, 180.0) * PI / 180.0;
        let elev_rad = elevation_deg.clamp(-90.0, 90.0) * PI / 180.0;
        let dist = distance_meters.max(0.1);

        // Distance inverse-square attenuation + 1.0 baseline
        let dist_gain = (1.0 / dist).clamp(0.01, 1.0);

        // Woodworth spherical head model for ITD (Interaural Time Difference):
        // theta is azimuth from front axis. Positive theta = sound on left.
        let theta = azim_rad * elev_rad.cos();
        let max_itd_secs = self.head_radius / self.speed_of_sound;

        let (left_delay_secs, right_delay_secs) = if theta >= 0.0 {
            // Sound is to the left: Left ear arrives first, Right ear is delayed by (theta + sin(theta))
            let itd = max_itd_secs * (theta + theta.sin());
            (0.0, itd)
        } else {
            // Sound is to the right: Right ear arrives first, Left ear is delayed
            let abs_theta = -theta;
            let itd = max_itd_secs * (abs_theta + abs_theta.sin());
            (itd, 0.0)
        };

        let left_delay_samples = (left_delay_secs * self.sample_rate).clamp(0.0, 120.0);
        let right_delay_samples = (right_delay_secs * self.sample_rate).clamp(0.0, 120.0);

        // Interaural Level Difference (ILD) Head Shadow factor:
        // Sound source on one side is attenuated on the contralateral ear
        let left_ild_gain = if theta >= 0.0 {
            1.0 + 0.3 * theta.sin()
        } else {
            (1.0 - 0.45 * (-theta).sin()).max(0.2)
        };

        let right_ild_gain = if theta <= 0.0 {
            1.0 + 0.3 * (-theta).sin()
        } else {
            (1.0 - 0.45 * theta.sin()).max(0.2)
        };

        // Shadow low-pass filter coefficient for contralateral acoustic shadow
        let shadow_alpha = 0.35f32;
        let buf_len = self.left_delay_buf.len();

        let count = mono_in.len().min(stereo_out.len() / 2);

        for i in 0..count {
            let input = mono_in[i] * dist_gain;

            // Write into ring buffers
            self.left_delay_buf[self.write_idx] = input;
            self.right_delay_buf[self.write_idx] = input;

            // Read delayed samples with linear fractional interpolation
            let left_sample = read_fractional_delay(
                &self.left_delay_buf,
                self.write_idx,
                left_delay_samples,
                buf_len,
            );
            let right_sample = read_fractional_delay(
                &self.right_delay_buf,
                self.write_idx,
                right_delay_samples,
                buf_len,
            );

            // Apply acoustic shadow filter
            self.left_shadow_state = (1.0 - shadow_alpha) * self.left_shadow_state
                + shadow_alpha * (left_sample * left_ild_gain);
            self.right_shadow_state = (1.0 - shadow_alpha) * self.right_shadow_state
                + shadow_alpha * (right_sample * right_ild_gain);

            stereo_out[i * 2] = self.left_shadow_state;
            stereo_out[i * 2 + 1] = self.right_shadow_state;

            self.write_idx = (self.write_idx + 1) % buf_len;
        }
    }

    /// Decodes an Ambisonics B-format stream into standard stereo headphones / speakers.
    /// L = W + (sqrt(2)/2)*Y,  R = W - (sqrt(2)/2)*Y
    pub fn decode_bformat_to_stereo(bformat: &[BFormatFrame], stereo_out: &mut [f32]) {
        let count = bformat.len().min(stereo_out.len() / 2);
        let coeff = std::f32::consts::FRAC_1_SQRT_2;

        for (i, frame) in bformat.iter().take(count).enumerate() {
            let left = frame.w + coeff * frame.y;
            let right = frame.w - coeff * frame.y;

            stereo_out[i * 2] = left;
            stereo_out[i * 2 + 1] = right;
        }
    }

    /// Decodes an Ambisonics B-format stream into 5.1 surround sound:
    /// [L, R, C, LFE, Ls, Rs]
    pub fn decode_bformat_to_5_1(bformat: &[BFormatFrame], surround_out: &mut [f32]) {
        let count = bformat.len().min(surround_out.len() / 6);
        let inv_sqrt2 = std::f32::consts::FRAC_1_SQRT_2;

        for (i, f) in bformat.iter().take(count).enumerate() {
            let l = f.w + inv_sqrt2 * (f.x + f.y);
            let r = f.w + inv_sqrt2 * (f.x - f.y);
            let c = f.w + f.x;
            let lfe = f.w * 0.5; // low frequency omni component
            let ls = f.w + inv_sqrt2 * (-f.x + f.y);
            let rs = f.w + inv_sqrt2 * (-f.x - f.y);

            let offset = i * 6;
            surround_out[offset] = l;
            surround_out[offset + 1] = r;
            surround_out[offset + 2] = c;
            surround_out[offset + 3] = lfe;
            surround_out[offset + 4] = ls;
            surround_out[offset + 5] = rs;
        }
    }
}

fn read_fractional_delay(buf: &[f32], write_idx: usize, delay: f32, buf_len: usize) -> f32 {
    let int_delay = delay.floor() as usize;
    let frac = delay - delay.floor();

    let idx1 = (write_idx + buf_len - int_delay) % buf_len;
    let idx2 = (idx1 + buf_len - 1) % buf_len;

    (1.0 - frac) * buf[idx1] + frac * buf[idx2]
}
