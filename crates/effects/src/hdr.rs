use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// High Dynamic Range (HDR) non-linear electro-optical transfer functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HdrTransferFunction {
    /// SMPTE ST 2084 Perceptual Quantizer (PQ), standard for HDR10 and Dolby Vision.
    Pq,
    /// ARIB STD-B67 Hybrid Log-Gamma (HLG), standard for live broadcast HDR.
    Hlg,
    /// Standard Linear radiometric response.
    Linear,
}

/// Tone mapping algorithms mapping high-dynamic range radiance to SDR standard displays (Rec.709).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToneMappingAlgorithm {
    /// Academy Color Encoding System (ACES) filmic S-curve tone curve.
    AcesFilmic,
    /// Extended Reinhard operator with white luminance parameter.
    ReinhardExtended,
    /// Exponential photographic soft shoulder compression.
    PhotographicKnee,
}

/// Parameters configuring the HDR tone mapping pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HdrToneMapParams {
    pub transfer_function: HdrTransferFunction,
    pub algorithm: ToneMappingAlgorithm,
    /// Exposure adjustment in stops (-4.0 to +4.0, default 0.0).
    pub exposure_stops: f32,
    /// Target display peak luminance in nits (e.g. 100.0 for standard SDR, 300.0 for high-bright SDR).
    pub target_nits: f32,
    /// Extended Reinhard white point luminance (default 3.0).
    pub white_point: f32,
}

impl Default for HdrToneMapParams {
    fn default() -> Self {
        Self {
            transfer_function: HdrTransferFunction::Pq,
            algorithm: ToneMappingAlgorithm::AcesFilmic,
            exposure_stops: 0.0,
            target_nits: 100.0,
            white_point: 3.0,
        }
    }
}

/// Converts SMPTE ST 2084 (PQ) normalized non-linear code [0.0, 1.0] to linear luminance in nits [0, 10000].
pub fn pq_eotf(n: f32) -> f32 {
    let n = n.clamp(0.0, 1.0);
    if n <= 0.0 {
        return 0.0;
    }
    // ST 2084 constants
    const M1: f32 = 2610.0 / 16384.0;
    const M2: f32 = (2523.0 / 4096.0) * 128.0;
    const C1: f32 = 3424.0 / 4096.0;
    const C2: f32 = (2413.0 / 4096.0) * 32.0;
    const C3: f32 = (2392.0 / 4096.0) * 32.0;

    let n_pow_inv_m2 = n.powf(1.0 / M2);
    let numerator = (n_pow_inv_m2 - C1).max(0.0);
    let denominator = C2 - C3 * n_pow_inv_m2;

    if denominator <= 0.0 {
        return 10000.0;
    }

    (numerator / denominator).powf(1.0 / M1) * 10000.0
}

/// Converts linear luminance in nits [0, 10000] to SMPTE ST 2084 (PQ) normalized non-linear code [0.0, 1.0].
pub fn pq_inverse_eotf(nits: f32) -> f32 {
    let nits = nits.clamp(0.0, 10000.0);
    if nits <= 0.0 {
        return 0.0;
    }
    const M1: f32 = 2610.0 / 16384.0;
    const M2: f32 = (2523.0 / 4096.0) * 128.0;
    const C1: f32 = 3424.0 / 4096.0;
    const C2: f32 = (2413.0 / 4096.0) * 32.0;
    const C3: f32 = (2392.0 / 4096.0) * 32.0;

    let y = (nits / 10000.0).powf(M1);
    let num = C1 + C2 * y;
    let den = 1.0 + C3 * y;

    (num / den).powf(M2)
}

/// Converts ARIB STD-B67 (HLG) non-linear code to linear scene light.
pub fn hlg_inverse_oetf(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    const A: f32 = 0.17883277;
    const B: f32 = 0.28466892; // 1 - 4a
    const C: f32 = 0.559_910_7; // 0.5 - a*ln(4a)

    if v <= 0.5 {
        (v * v) / 3.0
    } else {
        (((v - C) / A).exp() + B) / 12.0
    }
}

/// Krzysztof Narkowicz fitted ACES tone reproduction curve.
pub fn aces_filmic(x: f32) -> f32 {
    let x = x.max(0.0);
    const A: f32 = 2.51;
    const B: f32 = 0.03;
    const C: f32 = 2.43;
    const D: f32 = 0.59;
    const E: f32 = 0.14;

    ((x * (A * x + B)) / (x * (C * x + D) + E)).clamp(0.0, 1.0)
}

/// Extended Reinhard tone mapping with customizable white point clipping.
pub fn reinhard_extended(x: f32, white_point: f32) -> f32 {
    let x = x.max(0.0);
    let w_sq = (white_point * white_point).max(0.01);
    ((x * (1.0 + x / w_sq)) / (1.0 + x)).clamp(0.0, 1.0)
}

/// Applies HDR Tone Mapping to an RGBA pixel buffer in-place using Rayon parallel scanlines.
pub fn apply_hdr_tonemap(pixels: &mut [u8], params: &HdrToneMapParams) {
    let exposure_scale = 2.0f32.powf(params.exposure_stops);
    let norm_scale = 1.0 / params.target_nits.max(50.0);
    let white_pt = params.white_point.max(1.0);

    pixels.par_chunks_exact_mut(4).for_each(|pixel| {
        let r_in = pixel[0] as f32 / 255.0;
        let g_in = pixel[1] as f32 / 255.0;
        let b_in = pixel[2] as f32 / 255.0;

        // 1. Decode EOTF to linear luminance
        let (r_lin, g_lin, b_lin) = match params.transfer_function {
            HdrTransferFunction::Pq => (
                pq_eotf(r_in) * norm_scale,
                pq_eotf(g_in) * norm_scale,
                pq_eotf(b_in) * norm_scale,
            ),
            HdrTransferFunction::Hlg => (
                hlg_inverse_oetf(r_in) * 10.0,
                hlg_inverse_oetf(g_in) * 10.0,
                hlg_inverse_oetf(b_in) * 10.0,
            ),
            HdrTransferFunction::Linear => (r_in * 4.0, g_in * 4.0, b_in * 4.0),
        };

        // 2. Apply exposure scale
        let r_exp = r_lin * exposure_scale;
        let g_exp = g_lin * exposure_scale;
        let b_exp = b_lin * exposure_scale;

        // 3. Apply Tone Mapping operator
        let (r_mapped, g_mapped, b_mapped) = match params.algorithm {
            ToneMappingAlgorithm::AcesFilmic => {
                (aces_filmic(r_exp), aces_filmic(g_exp), aces_filmic(b_exp))
            }
            ToneMappingAlgorithm::ReinhardExtended => (
                reinhard_extended(r_exp, white_pt),
                reinhard_extended(g_exp, white_pt),
                reinhard_extended(b_exp, white_pt),
            ),
            ToneMappingAlgorithm::PhotographicKnee => {
                let soft = |x: f32| 1.0 - (-x).exp();
                (soft(r_exp), soft(g_exp), soft(b_exp))
            }
        };

        // 4. Quantize to SDR display sRGB/Rec.709 gamma ~2.2
        let to_srgb = |c: f32| -> u8 {
            let clamped = c.clamp(0.0, 1.0);
            (clamped.powf(1.0 / 2.2) * 255.0).round() as u8
        };

        pixel[0] = to_srgb(r_mapped);
        pixel[1] = to_srgb(g_mapped);
        pixel[2] = to_srgb(b_mapped);
    });
}
