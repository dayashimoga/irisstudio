//! Optical Flow Frame Interpolation Engine.
//!
//! Provides bidirectional motion vector field calculation, forward-backward
//! consistency checking for occlusion detection, and motion-compensated frame synthesis
//! for cinema-grade slow-motion (2x, 4x, 8x) and framerate up-conversion.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a single motion vector between two video frames.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionVector {
    pub dx: f32,
    pub dy: f32,
    pub cost: f32,
    pub is_occluded: bool,
}

impl Default for MotionVector {
    fn default() -> Self {
        Self {
            dx: 0.0,
            dy: 0.0,
            cost: 0.0,
            is_occluded: false,
        }
    }
}

/// A 2D grid of motion vectors representing pixel block displacements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionField {
    pub width_blocks: usize,
    pub height_blocks: usize,
    pub block_size: usize,
    pub vectors: Vec<MotionVector>,
}

impl MotionField {
    pub fn new(width_blocks: usize, height_blocks: usize, block_size: usize) -> Self {
        let total = width_blocks * height_blocks;
        Self {
            width_blocks,
            height_blocks,
            block_size,
            vectors: vec![MotionVector::default(); total],
        }
    }

    #[inline]
    pub fn get(&self, bx: usize, by: usize) -> MotionVector {
        if bx < self.width_blocks && by < self.height_blocks {
            self.vectors[by * self.width_blocks + bx]
        } else {
            MotionVector::default()
        }
    }

    #[inline]
    pub fn set(&mut self, bx: usize, by: usize, vec: MotionVector) {
        if bx < self.width_blocks && by < self.height_blocks {
            self.vectors[by * self.width_blocks + bx] = vec;
        }
    }

    /// Bilinearly samples the motion vector at continuous pixel coordinates (px, py).
    pub fn sample_bilinear(&self, px: f32, py: f32) -> (f32, f32, bool) {
        if self.width_blocks == 0 || self.height_blocks == 0 || self.block_size == 0 {
            return (0.0, 0.0, false);
        }

        let bs = self.block_size as f32;
        let gx = (px / bs).clamp(0.0, (self.width_blocks - 1) as f32);
        let gy = (py / bs).clamp(0.0, (self.height_blocks - 1) as f32);

        let bx0 = gx.floor() as usize;
        let by0 = gy.floor() as usize;
        let bx1 = (bx0 + 1).min(self.width_blocks - 1);
        let by1 = (by0 + 1).min(self.height_blocks - 1);

        let fx = gx - bx0 as f32;
        let fy = gy - by0 as f32;

        let v00 = self.get(bx0, by0);
        let v10 = self.get(bx1, by0);
        let v01 = self.get(bx0, by1);
        let v11 = self.get(bx1, by1);

        let dx = (1.0 - fx) * (1.0 - fy) * v00.dx
            + fx * (1.0 - fy) * v10.dx
            + (1.0 - fx) * fy * v01.dx
            + fx * fy * v11.dx;

        let dy = (1.0 - fx) * (1.0 - fy) * v00.dy
            + fx * (1.0 - fy) * v10.dy
            + (1.0 - fx) * fy * v01.dy
            + fx * fy * v11.dy;

        let is_occluded = v00.is_occluded || v10.is_occluded || v01.is_occluded || v11.is_occluded;

        (dx, dy, is_occluded)
    }
}

/// Parameters configuring optical flow estimation and interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OpticalFlowConfig {
    pub block_size: usize,
    pub search_radius: i32,
    pub occlusion_threshold: f32,
}

impl Default for OpticalFlowConfig {
    fn default() -> Self {
        Self {
            block_size: 8,
            search_radius: 12,
            occlusion_threshold: 2.0,
        }
    }
}

/// Calculates the Sum of Absolute Differences (SAD) between two blocks.
#[inline]
fn compute_block_sad(
    frame_a: &[u8],
    frame_b: &[u8],
    width: usize,
    height: usize,
    pos_a: (usize, usize),
    pos_b: (usize, usize),
    block_size: usize,
) -> f32 {
    let (ax, ay) = pos_a;
    let (bx, by) = pos_b;
    let mut total_sad: u32 = 0;
    let b_w = block_size
        .min(width.saturating_sub(ax))
        .min(width.saturating_sub(bx));
    let b_h = block_size
        .min(height.saturating_sub(ay))
        .min(height.saturating_sub(by));

    if b_w == 0 || b_h == 0 {
        return f32::MAX;
    }

    for y in 0..b_h {
        let row_a = (ay + y) * width * 4;
        let row_b = (by + y) * width * 4;
        for x in 0..b_w {
            let idx_a = row_a + (ax + x) * 4;
            let idx_b = row_b + (bx + x) * 4;

            let dr = (frame_a[idx_a] as i32 - frame_b[idx_b] as i32).unsigned_abs();
            let dg = (frame_a[idx_a + 1] as i32 - frame_b[idx_b + 1] as i32).unsigned_abs();
            let db = (frame_a[idx_a + 2] as i32 - frame_b[idx_b + 2] as i32).unsigned_abs();

            total_sad += dr + dg + db;
        }
    }

    total_sad as f32
}

/// Estimates the motion vector field from `frame_from` to `frame_to`.
pub fn estimate_motion_field(
    frame_from: &[u8],
    frame_to: &[u8],
    width: usize,
    height: usize,
    config: &OpticalFlowConfig,
) -> MotionField {
    let bs = config.block_size;
    let radius = config.search_radius;
    let w_blocks = width.div_ceil(bs);
    let h_blocks = height.div_ceil(bs);

    let mut field = MotionField::new(w_blocks, h_blocks, bs);

    // Compute motion vectors in parallel across block rows
    let rows: Vec<Vec<MotionVector>> = (0..h_blocks)
        .into_par_iter()
        .map(|by| {
            let mut row = Vec::with_capacity(w_blocks);
            let src_y = by * bs;

            for bx in 0..w_blocks {
                let src_x = bx * bs;
                let mut best_sad = f32::MAX;
                let mut best_dx = 0;
                let mut best_dy = 0;

                for dy in -radius..=radius {
                    let target_y = src_y as i32 + dy;
                    if target_y < 0 || target_y as usize >= height {
                        continue;
                    }

                    for dx in -radius..=radius {
                        let target_x = src_x as i32 + dx;
                        if target_x < 0 || target_x as usize >= width {
                            continue;
                        }

                        let sad = compute_block_sad(
                            frame_from,
                            frame_to,
                            width,
                            height,
                            (src_x, src_y),
                            (target_x as usize, target_y as usize),
                            bs,
                        );

                        if sad < best_sad {
                            best_sad = sad;
                            best_dx = dx;
                            best_dy = dy;
                        }
                    }
                }

                // Sub-pixel parabolic interpolation refinement
                let mut refined_dx = best_dx as f32;
                let mut refined_dy = best_dy as f32;

                if best_dx.abs() < radius && best_dy.abs() < radius {
                    let target_x = src_x as i32 + best_dx;
                    let target_y = src_y as i32 + best_dy;

                    if target_x > 0
                        && (target_x as usize + 1) < width
                        && target_y > 0
                        && (target_y as usize + 1) < height
                    {
                        let sad_left = compute_block_sad(
                            frame_from,
                            frame_to,
                            width,
                            height,
                            (src_x, src_y),
                            ((target_x - 1) as usize, target_y as usize),
                            bs,
                        );
                        let sad_right = compute_block_sad(
                            frame_from,
                            frame_to,
                            width,
                            height,
                            (src_x, src_y),
                            ((target_x + 1) as usize, target_y as usize),
                            bs,
                        );
                        let denom_x = (sad_left - 2.0 * best_sad + sad_right).abs();
                        if denom_x > 1e-4 {
                            let sub_x = (sad_left - sad_right) / (2.0 * denom_x);
                            refined_dx += sub_x.clamp(-0.5, 0.5);
                        }

                        let sad_up = compute_block_sad(
                            frame_from,
                            frame_to,
                            width,
                            height,
                            (src_x, src_y),
                            (target_x as usize, (target_y - 1) as usize),
                            bs,
                        );
                        let sad_down = compute_block_sad(
                            frame_from,
                            frame_to,
                            width,
                            height,
                            (src_x, src_y),
                            (target_x as usize, (target_y + 1) as usize),
                            bs,
                        );
                        let denom_y = (sad_up - 2.0 * best_sad + sad_down).abs();
                        if denom_y > 1e-4 {
                            let sub_y = (sad_up - sad_down) / (2.0 * denom_y);
                            refined_dy += sub_y.clamp(-0.5, 0.5);
                        }
                    }
                }

                row.push(MotionVector {
                    dx: refined_dx,
                    dy: refined_dy,
                    cost: best_sad,
                    is_occluded: false,
                });
            }
            row
        })
        .collect();

    for (by, row) in rows.into_iter().enumerate() {
        for (bx, vec) in row.into_iter().enumerate() {
            field.set(bx, by, vec);
        }
    }

    field
}

/// Performs a forward-backward consistency check to detect occlusions.
pub fn check_forward_backward_consistency(
    forward_field: &mut MotionField,
    backward_field: &MotionField,
    threshold: f32,
) {
    let bs = forward_field.block_size as f32;

    for by in 0..forward_field.height_blocks {
        for bx in 0..forward_field.width_blocks {
            let fwd = forward_field.get(bx, by);

            let target_px = bx as f32 * bs + fwd.dx;
            let target_py = by as f32 * bs + fwd.dy;

            let (bwd_dx, bwd_dy, _) = backward_field.sample_bilinear(target_px, target_py);

            let err_x = fwd.dx + bwd_dx;
            let err_y = fwd.dy + bwd_dy;
            let consistency_err = (err_x * err_x + err_y * err_y).sqrt();

            if consistency_err > threshold {
                let mut updated = fwd;
                updated.is_occluded = true;
                forward_field.set(bx, by, updated);
            }
        }
    }
}

/// Bilinearly samples a color from an RGBA8 frame at continuous coordinates.
#[inline]
fn sample_frame_bilinear(frame: &[u8], width: usize, height: usize, px: f32, py: f32) -> [u8; 4] {
    let px = px.clamp(0.0, (width - 1) as f32);
    let py = py.clamp(0.0, (height - 1) as f32);

    let x0 = px.floor() as usize;
    let y0 = py.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = px - x0 as f32;
    let fy = py - y0 as f32;

    let idx00 = (y0 * width + x0) * 4;
    let idx10 = (y0 * width + x1) * 4;
    let idx01 = (y1 * width + x0) * 4;
    let idx11 = (y1 * width + x1) * 4;

    let mut out = [0u8; 4];
    for c in 0..4 {
        let c00 = frame[idx00 + c] as f32;
        let c10 = frame[idx10 + c] as f32;
        let c01 = frame[idx01 + c] as f32;
        let c11 = frame[idx11 + c] as f32;

        let val = (1.0 - fx) * (1.0 - fy) * c00
            + fx * (1.0 - fy) * c10
            + (1.0 - fx) * fy * c01
            + fx * fy * c11;

        out[c] = val.round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Interpolates a motion-compensated intermediate frame at time `t` (0.0 to 1.0)
/// between `frame_0` and `frame_1`.
#[allow(clippy::too_many_arguments)]
pub fn interpolate_frame(
    frame_0: &[u8],
    frame_1: &[u8],
    output: &mut [u8],
    width: usize,
    height: usize,
    forward_field: &MotionField,
    backward_field: &MotionField,
    t: f32,
) {
    let t = t.clamp(0.0, 1.0);
    assert_eq!(output.len(), width * height * 4);

    let stride = width * 4;

    output
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, scanline)| {
            for x in 0..width {
                let px = x as f32;
                let py = y as f32;

                let (fwd_dx, fwd_dy, fwd_occluded) = forward_field.sample_bilinear(px, py);
                let (bwd_dx, bwd_dy, bwd_occluded) = backward_field.sample_bilinear(px, py);

                // Motion-compensated source locations
                let src0_x = px - t * fwd_dx;
                let src0_y = py - t * fwd_dy;

                let src1_x = px - (1.0 - t) * bwd_dx;
                let src1_y = py - (1.0 - t) * bwd_dy;

                let c0 = sample_frame_bilinear(frame_0, width, height, src0_x, src0_y);
                let c1 = sample_frame_bilinear(frame_1, width, height, src1_x, src1_y);

                // Blend weights adapt dynamically to occlusions
                let (w0, w1) = if fwd_occluded && !bwd_occluded {
                    (0.15 * (1.0 - t), 0.85 + 0.15 * t)
                } else if !fwd_occluded && bwd_occluded {
                    (0.85 + 0.15 * (1.0 - t), 0.15 * t)
                } else {
                    (1.0 - t, t)
                };

                let norm = (w0 + w1).max(1e-4);
                let nw0 = w0 / norm;
                let nw1 = w1 / norm;

                let idx = x * 4;
                for c in 0..4 {
                    let val = nw0 * (c0[c] as f32) + nw1 * (c1[c] as f32);
                    scanline[idx + c] = val.round().clamp(0.0, 255.0) as u8;
                }
            }
        });
}

/// Generates a synthesized sequence of slow-motion frames between two input frames.
///
/// For example, `multiplier = 2` generates 1 intermediate frame at t=0.5.
/// `multiplier = 4` generates 3 intermediate frames at t=0.25, 0.5, 0.75.
pub fn generate_slow_motion_sequence(
    frame_0: &[u8],
    frame_1: &[u8],
    width: usize,
    height: usize,
    multiplier: usize,
    config: &OpticalFlowConfig,
) -> Vec<Vec<u8>> {
    if multiplier <= 1 {
        return vec![frame_0.to_vec()];
    }

    let mut forward_field = estimate_motion_field(frame_0, frame_1, width, height, config);
    let backward_field = estimate_motion_field(frame_1, frame_0, width, height, config);

    check_forward_backward_consistency(
        &mut forward_field,
        &backward_field,
        config.occlusion_threshold,
    );

    let mut intermediate_frames = Vec::with_capacity(multiplier);
    let step = 1.0 / (multiplier as f32);

    for i in 0..multiplier {
        let t = i as f32 * step;
        let mut out_frame = vec![0u8; width * height * 4];
        interpolate_frame(
            frame_0,
            frame_1,
            &mut out_frame,
            width,
            height,
            &forward_field,
            &backward_field,
            t,
        );
        intermediate_frames.push(out_frame);
    }

    intermediate_frames
}
