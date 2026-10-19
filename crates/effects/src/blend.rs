use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
}

#[inline]
pub fn blend_pixel(dst: [u8; 4], src: [u8; 4], mode: BlendMode, opacity: f32) -> [u8; 4] {
    let alpha_src = (src[3] as f32 / 255.0) * opacity.clamp(0.0, 1.0);
    if alpha_src <= 0.0 {
        return dst;
    }

    let alpha_dst = dst[3] as f32 / 255.0;
    let out_a = alpha_src + alpha_dst * (1.0 - alpha_src);
    if out_a <= 0.0 {
        return [0, 0, 0, 0];
    }

    let mut out = [0u8; 4];
    for c in 0..3 {
        let s = src[c] as f32 / 255.0;
        let d = dst[c] as f32 / 255.0;

        let blended = match mode {
            BlendMode::Normal => s,
            BlendMode::Multiply => s * d,
            BlendMode::Screen => 1.0 - (1.0 - s) * (1.0 - d),
            BlendMode::Overlay => {
                if d < 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::Darken => s.min(d),
            BlendMode::Lighten => s.max(d),
            BlendMode::ColorDodge => {
                if s >= 1.0 {
                    1.0
                } else {
                    (d / (1.0 - s)).min(1.0)
                }
            }
            BlendMode::ColorBurn => {
                if s <= 0.0 {
                    0.0
                } else {
                    (1.0 - (1.0 - d) / s).max(0.0)
                }
            }
            BlendMode::SoftLight => (1.0 - 2.0 * s) * d * d + 2.0 * s * d,
            BlendMode::HardLight => {
                if s < 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::Difference => (s - d).abs(),
            BlendMode::Exclusion => s + d - 2.0 * s * d,
        };

        // Standard compositing equation: (Blended * alpha_src + d * alpha_dst * (1 - alpha_src)) / out_a
        let val = (blended * alpha_src + d * alpha_dst * (1.0 - alpha_src)) / out_a;
        out[c] = (val * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    out[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    out
}

pub fn blend_buffers(dst: &mut [u8], src: &[u8], mode: BlendMode, opacity: f32) {
    assert_eq!(dst.len(), src.len());
    dst.par_chunks_mut(4)
        .zip(src.par_chunks(4))
        .for_each(|(d, s)| {
            let out = blend_pixel(
                [d[0], d[1], d[2], d[3]],
                [s[0], s[1], s[2], s[3]],
                mode,
                opacity,
            );
            d.copy_from_slice(&out);
        });
}
