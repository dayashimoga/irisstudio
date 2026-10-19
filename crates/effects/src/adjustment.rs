use iris_core::ColorSpace;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentParameters {
    pub exposure: f32,    // -5.0 to +5.0 EV
    pub contrast: f32,    // -100.0 to +100.0
    pub temperature: f32, // -100.0 (Cool) to +100.0 (Warm)
    pub tint: f32,        // -100.0 (Green) to +100.0 (Magenta)
    pub highlights: f32,  // -100.0 to +100.0
    pub shadows: f32,     // -100.0 to +100.0
    pub saturation: f32,  // -100.0 to +100.0
    pub vibrance: f32,    // -100.0 to +100.0
}

impl Default for AdjustmentParameters {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0.0,
            temperature: 0.0,
            tint: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            saturation: 0.0,
            vibrance: 0.0,
        }
    }
}

pub fn apply_adjustments(pixels: &mut [u8], params: &AdjustmentParameters) {
    if params == &AdjustmentParameters::default() {
        return; // Fast path identity
    }

    let exp_mult = 2.0f32.powf(params.exposure);
    let contrast_factor = (params.contrast / 100.0 + 1.0).max(0.0);
    let sat_mult = (params.saturation / 100.0 + 1.0).max(0.0);
    let vib_mult = params.vibrance / 100.0;

    let temp_r = 1.0 + params.temperature * 0.003;
    let temp_b = 1.0 - params.temperature * 0.003;
    let tint_g = 1.0 - params.tint * 0.003;

    // Parallel scanline or chunk processing with Rayon
    pixels.par_chunks_mut(4).for_each(|pixel| {
        let mut r = pixel[0] as f32 / 255.0;
        let mut g = pixel[1] as f32 / 255.0;
        let mut b = pixel[2] as f32 / 255.0;
        let a = pixel[3];

        // Linearize sRGB
        r = ColorSpace::srgb_to_linear(r);
        g = ColorSpace::srgb_to_linear(g);
        b = ColorSpace::srgb_to_linear(b);

        // 1. Exposure
        r *= exp_mult;
        g *= exp_mult;
        b *= exp_mult;

        // 2. White Balance / Tint
        r *= temp_r;
        b *= temp_b;
        g *= tint_g;

        // 3. Contrast (pivot around middle-grey 0.18 linear)
        r = ((r - 0.18) * contrast_factor + 0.18).max(0.0);
        g = ((g - 0.18) * contrast_factor + 0.18).max(0.0);
        b = ((b - 0.18) * contrast_factor + 0.18).max(0.0);

        // 4. Highlights / Shadows recovery
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if params.highlights != 0.0 {
            let h_factor = (luma * (params.highlights / 100.0) * 0.5).max(-1.0);
            r += r * h_factor;
            g += g * h_factor;
            b += b * h_factor;
        }
        if params.shadows != 0.0 {
            let s_factor = ((1.0 - luma) * (params.shadows / 100.0) * 0.5).max(-1.0);
            r += r * s_factor;
            g += g * s_factor;
            b += b * s_factor;
        }

        // Delinearize back to sRGB for perceptual saturation
        r = ColorSpace::linear_to_srgb(r.clamp(0.0, 1.0));
        g = ColorSpace::linear_to_srgb(g.clamp(0.0, 1.0));
        b = ColorSpace::linear_to_srgb(b.clamp(0.0, 1.0));

        // 5. Saturation & Vibrance
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let sat = if max_c == 0.0 {
            0.0
        } else {
            (max_c - min_c) / max_c
        };

        let perceived_luma = 0.299 * r + 0.587 * g + 0.114 * b;

        // Vibrance: increases saturation more on lower-saturated pixels
        let vib_boost = (1.0 - sat) * vib_mult;
        let total_sat = (sat_mult + vib_boost).max(0.0);

        r = perceived_luma + (r - perceived_luma) * total_sat;
        g = perceived_luma + (g - perceived_luma) * total_sat;
        b = perceived_luma + (b - perceived_luma) * total_sat;

        pixel[0] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
        pixel[1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
        pixel[2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
        pixel[3] = a;
    });
}
