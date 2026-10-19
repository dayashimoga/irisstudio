//! Smart Edge-Detection Rotoscoping & Background Segmentation Engine.
//!
//! Provides contrast-aware edge contour extraction, seed-based interactive
//! foreground/background region growing, and bilateral edge refinement
//! for non-destructive subject isolation and matte generation.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Interactive seed point marking a pixel as definite foreground or background.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeedPoint {
    pub x: u32,
    pub y: u32,
    pub is_foreground: bool,
}

/// Bounding rectangle constraining the region of interest for smart matting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ContourBoundingBox {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

/// Configuration parameters for smart rotoscoping and background segmentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartMaskParams {
    /// Edge sensitivity threshold [0.01 .. 1.0]. Lower values catch faint edges.
    pub contrast_threshold: f32,
    /// Color tolerance sigma [0.01 .. 1.0]. Controls spectral distance falloff.
    pub color_tolerance: f32,
    /// Edge feathering radius in pixels [0 .. 32].
    pub feather_radius: usize,
    /// Invert the resulting mask (background becomes opaque, foreground transparent).
    pub invert: bool,
    /// Number of geodesic propagation iterations [1 .. 10].
    pub iterations: usize,
}

impl Default for SmartMaskParams {
    fn default() -> Self {
        Self {
            contrast_threshold: 0.15,
            color_tolerance: 0.25,
            feather_radius: 2,
            invert: false,
            iterations: 3,
        }
    }
}

/// Computes the normalized gradient magnitude field using 3x3 Sobel operators.
pub fn compute_edge_gradient_field(src_rgba: &[u8], width: usize, height: usize) -> Vec<f32> {
    let mut gradients = vec![0.0f32; width * height];
    let stride = width * 4;

    // Parallel scanline gradient computation
    gradients
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            if y == 0 || y + 1 >= height {
                return;
            }

            for (x, item) in row.iter_mut().enumerate().take(width - 1).skip(1) {
                // Compute luma for 3x3 neighborhood
                let luma = |px: usize, py: usize| -> f32 {
                    let idx = py * stride + px * 4;
                    0.2126 * (src_rgba[idx] as f32)
                        + 0.7152 * (src_rgba[idx + 1] as f32)
                        + 0.0722 * (src_rgba[idx + 2] as f32)
                };

                let p00 = luma(x - 1, y - 1);
                let p10 = luma(x, y - 1);
                let p20 = luma(x + 1, y - 1);

                let p01 = luma(x - 1, y);
                let p21 = luma(x + 1, y);

                let p02 = luma(x - 1, y + 1);
                let p12 = luma(x, y + 1);
                let p22 = luma(x + 1, y + 1);

                // Sobel filters
                let gx = (p20 + 2.0 * p21 + p22) - (p00 + 2.0 * p01 + p02);
                let gy = (p02 + 2.0 * p12 + p22) - (p00 + 2.0 * p10 + p20);

                let mag = (gx * gx + gy * gy).sqrt() / 1442.0; // Normalized by max gradient 4*255*sqrt(2)
                *item = mag.clamp(0.0, 1.0);
            }
        });

    gradients
}

/// Generates an 8-bit alpha matte ([0 .. 255]) segmenting foreground from background.
pub fn generate_smart_mask(
    src_rgba: &[u8],
    width: usize,
    height: usize,
    seeds: &[SeedPoint],
    bbox: Option<ContourBoundingBox>,
    params: &SmartMaskParams,
) -> Vec<u8> {
    assert_eq!(src_rgba.len(), width * height * 4);

    let total_pixels = width * height;
    let mut probability = vec![0.5f32; total_pixels];
    let gradients = compute_edge_gradient_field(src_rgba, width, height);

    let (min_x, min_y, max_x, max_y) = match bbox {
        Some(b) => (
            (b.min_x as usize).min(width - 1),
            (b.min_y as usize).min(height - 1),
            (b.max_x as usize).min(width - 1),
            (b.max_y as usize).min(height - 1),
        ),
        None => (0, 0, width - 1, height - 1),
    };

    // If bounding box is specified, pixels outside are guaranteed background
    if bbox.is_some() {
        for y in 0..height {
            for x in 0..width {
                if x < min_x || x > max_x || y < min_y || y > max_y {
                    probability[y * width + x] = 0.0;
                }
            }
        }
    }

    // Initialize seed points
    let mut fg_colors: Vec<[f32; 3]> = Vec::new();
    let mut bg_colors: Vec<[f32; 3]> = Vec::new();

    for seed in seeds {
        let sx = (seed.x as usize).min(width - 1);
        let sy = (seed.y as usize).min(height - 1);
        let idx = sy * width + sx;
        let c_idx = idx * 4;

        let col = [
            src_rgba[c_idx] as f32 / 255.0,
            src_rgba[c_idx + 1] as f32 / 255.0,
            src_rgba[c_idx + 2] as f32 / 255.0,
        ];

        if seed.is_foreground {
            probability[idx] = 1.0;
            fg_colors.push(col);
        } else {
            probability[idx] = 0.0;
            bg_colors.push(col);
        }
    }

    // Default reference color seeds if none provided
    if fg_colors.is_empty() {
        // Sample center pixel
        let cx = (min_x + max_x) / 2;
        let cy = (min_y + max_y) / 2;
        let c_idx = (cy * width + cx) * 4;
        fg_colors.push([
            src_rgba[c_idx] as f32 / 255.0,
            src_rgba[c_idx + 1] as f32 / 255.0,
            src_rgba[c_idx + 2] as f32 / 255.0,
        ]);
        probability[cy * width + cx] = 1.0;
    }
    if bg_colors.is_empty() {
        // Sample corner pixel
        let c_idx = (min_y * width + min_x) * 4;
        bg_colors.push([
            src_rgba[c_idx] as f32 / 255.0,
            src_rgba[c_idx + 1] as f32 / 255.0,
            src_rgba[c_idx + 2] as f32 / 255.0,
        ]);
        probability[min_y * width + min_x] = 0.0;
    }

    let color_sigma = params.color_tolerance.clamp(0.01, 1.0);
    let edge_barrier = params.contrast_threshold.clamp(0.01, 1.0);

    // Initial color affinity pass
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let idx = y * width + x;
            if probability[idx] == 1.0 || probability[idx] == 0.0 {
                continue;
            }

            let c_idx = idx * 4;
            let p_col = [
                src_rgba[c_idx] as f32 / 255.0,
                src_rgba[c_idx + 1] as f32 / 255.0,
                src_rgba[c_idx + 2] as f32 / 255.0,
            ];

            let mut min_dist_fg = f32::MAX;
            for fg in &fg_colors {
                let d = (p_col[0] - fg[0]).powi(2)
                    + (p_col[1] - fg[1]).powi(2)
                    + (p_col[2] - fg[2]).powi(2);
                if d < min_dist_fg {
                    min_dist_fg = d;
                }
            }

            let mut min_dist_bg = f32::MAX;
            for bg in &bg_colors {
                let d = (p_col[0] - bg[0]).powi(2)
                    + (p_col[1] - bg[1]).powi(2)
                    + (p_col[2] - bg[2]).powi(2);
                if d < min_dist_bg {
                    min_dist_bg = d;
                }
            }

            let p_fg = (-min_dist_fg / (2.0 * color_sigma * color_sigma)).exp();
            let p_bg = (-min_dist_bg / (2.0 * color_sigma * color_sigma)).exp();
            let total = p_fg + p_bg;
            if total > 1e-5 {
                probability[idx] = p_fg / total;
            }
        }
    }

    // Iterative geodesic spatial relaxation with edge boundary barrier
    let iters = params.iterations.clamp(1, 10);
    for _ in 0..iters {
        let prev_prob = probability.clone();

        for y in min_y + 1..max_y {
            for x in min_x + 1..max_x {
                let idx = y * width + x;
                let grad = gradients[idx];

                // Strong edges act as barriers against spatial propagation
                if grad > edge_barrier {
                    continue;
                }

                let edge_weight = (1.0 - grad / edge_barrier).clamp(0.0, 1.0);

                let n_sum = prev_prob[idx - 1]
                    + prev_prob[idx + 1]
                    + prev_prob[idx - width]
                    + prev_prob[idx + width];
                let n_avg = n_sum * 0.25;

                probability[idx] =
                    (1.0 - 0.4 * edge_weight) * prev_prob[idx] + (0.4 * edge_weight) * n_avg;
            }
        }
    }

    // Guided feathering blur pass
    let mut matte = vec![0u8; total_pixels];
    let radius = params.feather_radius;

    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0f32;
            let mut count = 0.0f32;

            let r_i32 = radius as i32;
            for dy in -r_i32..=r_i32 {
                let py = (y as i32 + dy).clamp(0, (height - 1) as i32) as usize;
                for dx in -r_i32..=r_i32 {
                    let px = (x as i32 + dx).clamp(0, (width - 1) as i32) as usize;
                    sum += probability[py * width + px];
                    count += 1.0;
                }
            }

            let mut alpha = (sum / count).clamp(0.0, 1.0);
            if params.invert {
                alpha = 1.0 - alpha;
            }

            matte[y * width + x] = (alpha * 255.0).round() as u8;
        }
    }

    matte
}

/// Applies an alpha matte to an input RGBA buffer, producing an isolated foreground.
pub fn apply_smart_mask(
    src_rgba: &[u8],
    mask: &[u8],
    out_rgba: &mut [u8],
    width: usize,
    height: usize,
) {
    assert_eq!(src_rgba.len(), width * height * 4);
    assert_eq!(mask.len(), width * height);
    assert_eq!(out_rgba.len(), width * height * 4);

    out_rgba
        .par_chunks_exact_mut(4)
        .zip(src_rgba.par_chunks_exact(4))
        .zip(mask.par_iter())
        .for_each(|((dest, src), &alpha)| {
            dest[0] = src[0];
            dest[1] = src[1];
            dest[2] = src[2];
            // Modulate original alpha with mask alpha
            dest[3] = ((src[3] as u32 * alpha as u32) / 255) as u8;
        });
}
