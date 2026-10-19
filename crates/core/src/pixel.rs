use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8,
    Bgra8,
    Rgb8,
    RgbaF32,
    Yuv420p,
    Nv12,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 | Self::Bgra8 => 4,
            Self::Rgb8 => 3,
            Self::RgbaF32 => 16,
            Self::Yuv420p | Self::Nv12 => 2, // effective packed estimate
        }
    }
}
