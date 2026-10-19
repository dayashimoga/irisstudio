use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelsParameters {
    pub in_black: f32,  // 0.0 to 255.0
    pub in_gamma: f32,  // 0.1 to 10.0 (1.0 = linear)
    pub in_white: f32,  // 0.0 to 255.0
    pub out_black: f32, // 0.0 to 255.0
    pub out_white: f32, // 0.0 to 255.0
}

impl Default for LevelsParameters {
    fn default() -> Self {
        Self {
            in_black: 0.0,
            in_gamma: 1.0,
            in_white: 255.0,
            out_black: 0.0,
            out_white: 255.0,
        }
    }
}

impl LevelsParameters {
    pub fn build_lut(&self) -> [u8; 256] {
        let mut lut = [0u8; 256];
        let in_range = (self.in_white - self.in_black).max(0.001);
        let out_range = self.out_white - self.out_black;
        let inv_gamma = 1.0 / self.in_gamma.clamp(0.1, 10.0);

        for (i, slot) in lut.iter_mut().enumerate() {
            let input = i as f32;
            let normalized = ((input - self.in_black) / in_range).clamp(0.0, 1.0);
            let gamma_corrected = normalized.powf(inv_gamma);
            let output = self.out_black + gamma_corrected * out_range;
            *slot = output.round().clamp(0.0, 255.0) as u8;
        }

        lut
    }
}

pub fn apply_levels(pixels: &mut [u8], params: &LevelsParameters) {
    if params == &LevelsParameters::default() {
        return;
    }

    let lut = params.build_lut();

    pixels.par_chunks_mut(4).for_each(|pixel| {
        pixel[0] = lut[pixel[0] as usize];
        pixel[1] = lut[pixel[1] as usize];
        pixel[2] = lut[pixel[2] as usize];
        // Alpha unchanged
    });
}
