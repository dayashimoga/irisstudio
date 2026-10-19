use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorSpace {
    Srgb,
    LinearSrgb,
    DisplayP3,
    Rec709,
    Rec2020,
}

impl ColorSpace {
    /// Converts a non-linear sRGB channel (0.0 - 1.0) to linear light.
    #[inline]
    pub fn srgb_to_linear(v: f32) -> f32 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Converts a linear light channel (0.0 - 1.0) to non-linear sRGB.
    #[inline]
    pub fn linear_to_srgb(v: f32) -> f32 {
        if v <= 0.0031308 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    }
}
