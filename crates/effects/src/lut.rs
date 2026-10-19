use iris_core::IrisError;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LutPreset {
    TealAndOrange,
    WarmVintage,
    BleachBypass,
    FadedFilm,
    MonochromeContrast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lut3D {
    pub title: String,
    pub size: usize,
    pub domain_min: [f32; 3],
    pub domain_max: [f32; 3],
    pub table: Vec<[f32; 3]>,
}

impl Default for Lut3D {
    fn default() -> Self {
        Self::identity(17)
    }
}

impl Lut3D {
    /// Creates an identity 3D LUT of dimension `size` (standard sizes: 17, 33, 65).
    pub fn identity(size: usize) -> Self {
        let size = size.clamp(2, 128);
        let mut table = Vec::with_capacity(size * size * size);
        let inv = 1.0 / (size - 1) as f32;

        for b in 0..size {
            let blue = b as f32 * inv;
            for g in 0..size {
                let green = g as f32 * inv;
                for r in 0..size {
                    let red = r as f32 * inv;
                    table.push([red, green, blue]);
                }
            }
        }

        Self {
            title: "Identity".to_string(),
            size,
            domain_min: [0.0, 0.0, 0.0],
            domain_max: [1.0, 1.0, 1.0],
            table,
        }
    }

    /// Generates built-in professional LUT presets.
    pub fn preset(preset: LutPreset) -> Self {
        let size = 33;
        let mut lut = Self::identity(size);
        lut.title = match preset {
            LutPreset::TealAndOrange => "Teal & Orange",
            LutPreset::WarmVintage => "Warm Vintage",
            LutPreset::BleachBypass => "Bleach Bypass",
            LutPreset::FadedFilm => "Faded Film",
            LutPreset::MonochromeContrast => "Monochrome Contrast",
        }
        .to_string();

        for entry in lut.table.iter_mut() {
            let [r, g, b] = *entry;
            *entry = match preset {
                LutPreset::TealAndOrange => {
                    // Push shadows towards cyan/teal, push highlights towards orange/gold
                    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    let shadow_weight = (1.0 - luma).powf(1.8);
                    let highlight_weight = luma.powf(1.5);

                    let nr = (r + 0.28 * highlight_weight - 0.12 * shadow_weight).clamp(0.0, 1.0);
                    let ng = (g + 0.08 * highlight_weight + 0.04 * shadow_weight).clamp(0.0, 1.0);
                    let nb = (b - 0.20 * highlight_weight + 0.24 * shadow_weight).clamp(0.0, 1.0);

                    // S-curve contrast
                    let s_curve = |x: f32| x * x * (3.0 - 2.0 * x);
                    [s_curve(nr), s_curve(ng), s_curve(nb)]
                }
                LutPreset::WarmVintage => {
                    // Golden warm highlights, lifted milky blacks, slight green/yellow bias
                    let nr = (r * 1.08 + 0.04).clamp(0.0, 1.0);
                    let ng = (g * 0.98 + 0.03).clamp(0.0, 1.0);
                    let nb = (b * 0.84 + 0.05).clamp(0.0, 1.0);
                    [nr, ng, nb]
                }
                LutPreset::BleachBypass => {
                    // High contrast, reduced saturation, metallic highlights
                    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    let contrast = |c: f32| {
                        if c < 0.5 {
                            2.0 * c * c
                        } else {
                            1.0 - 2.0 * (1.0 - c) * (1.0 - c)
                        }
                    };
                    let desat = 0.55;
                    let nr = r * (1.0 - desat) + luma * desat;
                    let ng = g * (1.0 - desat) + luma * desat;
                    let nb = b * (1.0 - desat) + luma * desat;
                    [contrast(nr), contrast(ng), contrast(nb)]
                }
                LutPreset::FadedFilm => {
                    // Lifted black floor, lowered white ceiling, mellow shadows
                    let lift = 0.08;
                    let crush = 0.94;
                    let nr = lift + r * (crush - lift);
                    let ng = lift + g * (crush - lift);
                    let nb = lift + b * (crush - lift);
                    [nr, ng, nb]
                }
                LutPreset::MonochromeContrast => {
                    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    let contrast = luma * luma * (3.0 - 2.0 * luma);
                    [contrast, contrast, contrast]
                }
            };
        }

        lut
    }

    /// Parses an Adobe `.cube` file format string.
    pub fn parse_cube(content: &str) -> Result<Self, IrisError> {
        let mut title = "Imported 3D LUT".to_string();
        let mut size: Option<usize> = None;
        let mut domain_min = [0.0f32, 0.0, 0.0];
        let mut domain_max = [1.0f32, 1.0, 1.0];
        let mut table = Vec::new();

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(rest) = line.strip_prefix("TITLE") {
                let clean = rest.trim().trim_matches('"');
                if !clean.is_empty() {
                    title = clean.to_string();
                }
                continue;
            }

            if let Some(rest) = line.strip_prefix("LUT_3D_SIZE") {
                let s = rest
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| IrisError::InvalidParameter("Invalid LUT_3D_SIZE value".into()))?;
                if !(2..=128).contains(&s) {
                    return Err(IrisError::InvalidParameter(format!(
                        "LUT dimension {} outside supported range 2-128",
                        s
                    )));
                }
                size = Some(s);
                table.reserve(s * s * s);
                continue;
            }

            if let Some(rest) = line.strip_prefix("DOMAIN_MIN") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() == 3 {
                    domain_min[0] = parts[0].parse().unwrap_or(0.0);
                    domain_min[1] = parts[1].parse().unwrap_or(0.0);
                    domain_min[2] = parts[2].parse().unwrap_or(0.0);
                }
                continue;
            }

            if let Some(rest) = line.strip_prefix("DOMAIN_MAX") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() == 3 {
                    domain_max[0] = parts[0].parse().unwrap_or(1.0);
                    domain_max[1] = parts[1].parse().unwrap_or(1.0);
                    domain_max[2] = parts[2].parse().unwrap_or(1.0);
                }
                continue;
            }

            // Data line with 3 RGB floating point components
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>(),
                    parts[2].parse::<f32>(),
                ) {
                    table.push([r, g, b]);
                }
            }
        }

        let size = size.ok_or_else(|| {
            IrisError::InvalidParameter("Missing LUT_3D_SIZE declaration in .cube file".into())
        })?;

        let expected_count = size * size * size;
        if table.len() != expected_count {
            return Err(IrisError::InvalidParameter(format!(
                "Incomplete LUT data: expected {} entries for size {}, found {}",
                expected_count,
                size,
                table.len()
            )));
        }

        Ok(Self {
            title,
            size,
            domain_min,
            domain_max,
            table,
        })
    }

    #[inline(always)]
    fn lut_index(&self, r_idx: usize, g_idx: usize, b_idx: usize) -> usize {
        r_idx + g_idx * self.size + b_idx * self.size * self.size
    }

    /// Performs fast tetrahedral interpolation for a single RGB coordinate in [0.0, 1.0].
    /// Tetrahedral interpolation partitions each cube into 6 tetrahedra, sampling only 4
    /// vertices per evaluation with optimal color volume continuity.
    #[inline]
    pub fn sample_tetrahedral(&self, r: f32, g: f32, b: f32) -> [f32; 3] {
        let max_idx = (self.size - 1) as f32;

        let d_r = (self.domain_max[0] - self.domain_min[0]).max(1e-6);
        let d_g = (self.domain_max[1] - self.domain_min[1]).max(1e-6);
        let d_b = (self.domain_max[2] - self.domain_min[2]).max(1e-6);

        let u = ((r - self.domain_min[0]) / d_r * max_idx).clamp(0.0, max_idx);
        let v = ((g - self.domain_min[1]) / d_g * max_idx).clamp(0.0, max_idx);
        let w = ((b - self.domain_min[2]) / d_b * max_idx).clamp(0.0, max_idx);

        let i = (u.floor() as usize).min(self.size - 2);
        let j = (v.floor() as usize).min(self.size - 2);
        let k = (w.floor() as usize).min(self.size - 2);

        let fx = u - i as f32;
        let fy = v - j as f32;
        let fz = w - k as f32;

        let p000 = self.table[self.lut_index(i, j, k)];
        let p111 = self.table[self.lut_index(i + 1, j + 1, k + 1)];

        if fx >= fy && fy >= fz {
            let p100 = self.table[self.lut_index(i + 1, j, k)];
            let p110 = self.table[self.lut_index(i + 1, j + 1, k)];
            [
                p000[0]
                    + fx * (p100[0] - p000[0])
                    + fy * (p110[0] - p100[0])
                    + fz * (p111[0] - p110[0]),
                p000[1]
                    + fx * (p100[1] - p000[1])
                    + fy * (p110[1] - p100[1])
                    + fz * (p111[1] - p110[1]),
                p000[2]
                    + fx * (p100[2] - p000[2])
                    + fy * (p110[2] - p100[2])
                    + fz * (p111[2] - p110[2]),
            ]
        } else if fx >= fz && fz >= fy {
            let p100 = self.table[self.lut_index(i + 1, j, k)];
            let p101 = self.table[self.lut_index(i + 1, j, k + 1)];
            [
                p000[0]
                    + fx * (p100[0] - p000[0])
                    + fz * (p101[0] - p100[0])
                    + fy * (p111[0] - p101[0]),
                p000[1]
                    + fx * (p100[1] - p000[1])
                    + fz * (p101[1] - p100[1])
                    + fy * (p111[1] - p101[1]),
                p000[2]
                    + fx * (p100[2] - p000[2])
                    + fz * (p101[2] - p100[2])
                    + fy * (p111[2] - p101[2]),
            ]
        } else if fz >= fx && fx >= fy {
            let p001 = self.table[self.lut_index(i, j, k + 1)];
            let p101 = self.table[self.lut_index(i + 1, j, k + 1)];
            [
                p000[0]
                    + fz * (p001[0] - p000[0])
                    + fx * (p101[0] - p001[0])
                    + fy * (p111[0] - p101[0]),
                p000[1]
                    + fz * (p001[1] - p000[1])
                    + fx * (p101[1] - p001[1])
                    + fy * (p111[1] - p001[1]),
                p000[2]
                    + fz * (p001[2] - p000[2])
                    + fx * (p101[2] - p001[2])
                    + fy * (p111[2] - p101[2]),
            ]
        } else if fy >= fx && fx >= fz {
            let p010 = self.table[self.lut_index(i, j + 1, k)];
            let p110 = self.table[self.lut_index(i + 1, j + 1, k)];
            [
                p000[0]
                    + fy * (p010[0] - p000[0])
                    + fx * (p110[0] - p010[0])
                    + fz * (p111[0] - p110[0]),
                p000[1]
                    + fy * (p010[1] - p000[1])
                    + fx * (p110[1] - p010[1])
                    + fz * (p111[1] - p110[1]),
                p000[2]
                    + fy * (p010[2] - p000[2])
                    + fx * (p110[2] - p010[2])
                    + fz * (p111[2] - p110[2]),
            ]
        } else if fy >= fz && fz >= fx {
            let p010 = self.table[self.lut_index(i, j + 1, k)];
            let p011 = self.table[self.lut_index(i, j + 1, k + 1)];
            [
                p000[0]
                    + fy * (p010[0] - p000[0])
                    + fz * (p011[0] - p010[0])
                    + fx * (p111[0] - p011[0]),
                p000[1]
                    + fy * (p010[1] - p000[1])
                    + fz * (p011[1] - p010[1])
                    + fx * (p111[1] - p011[1]),
                p000[2]
                    + fy * (p010[2] - p000[2])
                    + fz * (p011[2] - p010[2])
                    + fx * (p111[2] - p011[2]),
            ]
        } else {
            // fz >= fy && fy >= fx
            let p001 = self.table[self.lut_index(i, j, k + 1)];
            let p011 = self.table[self.lut_index(i, j + 1, k + 1)];
            [
                p000[0]
                    + fz * (p001[0] - p000[0])
                    + fy * (p011[0] - p001[0])
                    + fx * (p111[0] - p011[0]),
                p000[1]
                    + fz * (p001[1] - p000[1])
                    + fy * (p011[1] - p001[1])
                    + fx * (p111[1] - p011[1]),
                p000[2]
                    + fz * (p001[2] - p000[2])
                    + fy * (p011[2] - p001[2])
                    + fx * (p111[2] - p011[2]),
            ]
        }
    }

    /// Applies 3D LUT to an RGBA pixel buffer using Rayon parallel scanlines with configurable intensity.
    pub fn apply_rgba(&self, pixels: &mut [u8], intensity: f32) {
        let intensity = intensity.clamp(0.0, 1.0);
        if intensity <= 0.001 {
            return;
        }

        pixels.par_chunks_exact_mut(4).for_each(|chunk| {
            let r = chunk[0] as f32 / 255.0;
            let g = chunk[1] as f32 / 255.0;
            let b = chunk[2] as f32 / 255.0;

            let [out_r, out_g, out_b] = self.sample_tetrahedral(r, g, b);

            if intensity >= 0.999 {
                chunk[0] = (out_r * 255.0).round().clamp(0.0, 255.0) as u8;
                chunk[1] = (out_g * 255.0).round().clamp(0.0, 255.0) as u8;
                chunk[2] = (out_b * 255.0).round().clamp(0.0, 255.0) as u8;
            } else {
                let blended_r = r + (out_r - r) * intensity;
                let blended_g = g + (out_g - g) * intensity;
                let blended_b = b + (out_b - b) * intensity;

                chunk[0] = (blended_r * 255.0).round().clamp(0.0, 255.0) as u8;
                chunk[1] = (blended_g * 255.0).round().clamp(0.0, 255.0) as u8;
                chunk[2] = (blended_b * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        });
    }
}
