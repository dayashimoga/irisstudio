use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Parameters for real-time Chroma Keying (green/blue screen removal).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChromaKeyParams {
    /// Target key color in normalized RGB [0.0, 1.0] (default Green: [0.0, 1.0, 0.0]).
    pub key_color: [f32; 3],
    /// Color similarity cutoff threshold (0.0 to 1.0, default 0.35).
    pub similarity: f32,
    /// Edge smoothness feathering range (0.0 to 1.0, default 0.15).
    pub smoothness: f32,
    /// Spill suppression factor to remove green/blue fringe bounce (0.0 to 1.0, default 0.8).
    pub spill_suppression: f32,
}

impl Default for ChromaKeyParams {
    fn default() -> Self {
        Self {
            key_color: [0.0, 1.0, 0.0], // Pure green
            similarity: 0.35,
            smoothness: 0.15,
            spill_suppression: 0.8,
        }
    }
}

impl ChromaKeyParams {
    pub fn green_screen() -> Self {
        Self::default()
    }

    pub fn blue_screen() -> Self {
        Self {
            key_color: [0.0, 0.0, 1.0],
            similarity: 0.35,
            smoothness: 0.15,
            spill_suppression: 0.8,
        }
    }
}

/// Applies Chroma Keying to an RGBA pixel buffer in place using parallel scanline processing.
/// Modifies the alpha channel to transparent for key background and despills edge fringes.
pub fn apply_chroma_key(pixels: &mut [u8], params: &ChromaKeyParams) {
    let [kr, kg, kb] = params.key_color;
    let similarity = params.similarity.clamp(0.01, 1.0);
    let smoothness = params.smoothness.clamp(0.001, 1.0);
    let spill = params.spill_suppression.clamp(0.0, 1.0);

    let is_green_dominant = kg > kr && kg > kb;
    let is_blue_dominant = kb > kr && kb > kg;

    pixels.par_chunks_exact_mut(4).for_each(|chunk| {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        // Euclidean distance in normalized color space
        let dr = r - kr;
        let dg = g - kg;
        let db = b - kb;
        let dist = (dr * dr * 0.299 + dg * dg * 0.587 + db * db * 0.114).sqrt();

        // Calculate matte alpha
        let alpha = if dist <= similarity {
            0.0
        } else if dist >= similarity + smoothness {
            1.0
        } else {
            let t = (dist - similarity) / smoothness;
            // Smoothstep curve: 3t^2 - 2t^3
            t * t * (3.0 - 2.0 * t)
        };

        // Despill color suppression on semi-transparent and foreground edges
        let out_r = r;
        let mut out_g = g;
        let mut out_b = b;

        if spill > 0.0 {
            if is_green_dominant {
                let max_allowed_g = (r * 0.5 + b * 0.5) * (1.0 + (1.0 - spill) * 0.5);
                if g > max_allowed_g {
                    out_g = g * (1.0 - spill) + max_allowed_g * spill;
                }
            } else if is_blue_dominant {
                let max_allowed_b = (r * 0.5 + g * 0.5) * (1.0 + (1.0 - spill) * 0.5);
                if b > max_allowed_b {
                    out_b = b * (1.0 - spill) + max_allowed_b * spill;
                }
            }
        }

        chunk[0] = (out_r * 255.0).round().clamp(0.0, 255.0) as u8;
        chunk[1] = (out_g * 255.0).round().clamp(0.0, 255.0) as u8;
        chunk[2] = (out_b * 255.0).round().clamp(0.0, 255.0) as u8;

        // Modulate existing alpha
        let orig_alpha = chunk[3] as f32 / 255.0;
        chunk[3] = ((orig_alpha * alpha) * 255.0).round().clamp(0.0, 255.0) as u8;
    });
}
