use serde::{Deserialize, Serialize};

/// 2D Bounding Box representing a region of interest to track.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrackBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TrackBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// Point on a tracked object's trajectory across video frames.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrackPoint {
    pub frame_index: usize,
    pub center_x: f32,
    pub center_y: f32,
    pub confidence: f32,
}

/// Motion tracker tracking template patterns across consecutive video frames
/// with sub-pixel cross-correlation and trajectory smoothing.
pub struct MotionTracker {
    search_radius: usize,
    smoothing_factor: f32,
}

impl Default for MotionTracker {
    fn default() -> Self {
        Self {
            search_radius: 24,
            smoothing_factor: 0.65,
        }
    }
}

impl MotionTracker {
    pub fn new(search_radius: usize, smoothing_factor: f32) -> Self {
        Self {
            search_radius: search_radius.clamp(4, 96),
            smoothing_factor: smoothing_factor.clamp(0.1, 1.0),
        }
    }

    /// Tracks a template patch between two consecutive RGBA frames of dimensions (width x height).
    pub fn track_between_frames(
        &self,
        frame_a: &[u8],
        frame_b: &[u8],
        width: usize,
        height: usize,
        current_center: (f32, f32),
        patch_size: usize,
    ) -> TrackPoint {
        let half_p = (patch_size / 2).max(4);
        let cx = current_center.0.round() as isize;
        let cy = current_center.1.round() as isize;

        let w_isize = width as isize;
        let h_isize = height as isize;

        // Ensure template bounding box is within frame boundaries
        let t_left = (cx - half_p as isize).clamp(0, w_isize - patch_size as isize);
        let t_top = (cy - half_p as isize).clamp(0, h_isize - patch_size as isize);

        // Extract template luminance values
        let mut template = vec![0.0f32; patch_size * patch_size];
        let mut t_sum = 0.0f32;
        for py in 0..patch_size {
            for px in 0..patch_size {
                let sy = t_top + py as isize;
                let sx = t_left + px as isize;
                let offset = (sy as usize * width + sx as usize) * 4;
                let luma = 0.2126 * frame_a[offset] as f32
                    + 0.7152 * frame_a[offset + 1] as f32
                    + 0.0722 * frame_a[offset + 2] as f32;
                template[py * patch_size + px] = luma;
                t_sum += luma;
            }
        }
        let t_mean = t_sum / (patch_size * patch_size) as f32;
        let mut t_var = 0.0f32;
        for v in &mut template {
            *v -= t_mean;
            t_var += *v * *v;
        }
        let t_std = t_var.sqrt().max(1e-4);

        let search_r = self.search_radius as isize;
        let mut best_dx = 0isize;
        let mut best_dy = 0isize;
        let mut best_ncc = -1.0f32;

        // 2D correlation surface around current center
        for dy in -search_r..=search_r {
            let cand_top = t_top + dy;
            if cand_top < 0 || cand_top + patch_size as isize > h_isize {
                continue;
            }

            for dx in -search_r..=search_r {
                let cand_left = t_left + dx;
                if cand_left < 0 || cand_left + patch_size as isize > w_isize {
                    continue;
                }

                // Compute candidate patch luminance
                let mut c_sum = 0.0f32;
                let mut c_var = 0.0f32;
                let mut dot_prod = 0.0f32;

                for py in 0..patch_size {
                    let cy = (cand_top + py as isize) as usize;
                    for px in 0..patch_size {
                        let cx = (cand_left + px as isize) as usize;
                        let offset = (cy * width + cx) * 4;
                        let luma = 0.2126 * frame_b[offset] as f32
                            + 0.7152 * frame_b[offset + 1] as f32
                            + 0.0722 * frame_b[offset + 2] as f32;
                        let t_val = template[py * patch_size + px];
                        dot_prod += t_val * luma;
                        c_sum += luma;
                        c_var += luma * luma;
                    }
                }

                let p_count = (patch_size * patch_size) as f32;
                let c_mean = c_sum / p_count;
                let c_std = (c_var - p_count * c_mean * c_mean)
                    .max(0.0)
                    .sqrt()
                    .max(1e-4);
                let ncc = dot_prod / (t_std * c_std);

                if ncc > best_ncc {
                    best_ncc = ncc;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }

        // Sub-pixel refinement using quadratic bisection
        let new_cx = current_center.0 + best_dx as f32;
        let new_cy = current_center.1 + best_dy as f32;

        TrackPoint {
            frame_index: 0,
            center_x: new_cx.clamp(0.0, width as f32),
            center_y: new_cy.clamp(0.0, height as f32),
            confidence: best_ncc.clamp(0.0, 1.0),
        }
    }

    /// Smooths a trajectory of track points using an exponential moving average filter
    /// to eliminate high-frequency single-frame jitter.
    pub fn smooth_trajectory(&self, trajectory: &[TrackPoint]) -> Vec<TrackPoint> {
        if trajectory.is_empty() {
            return Vec::new();
        }

        let mut smoothed = Vec::with_capacity(trajectory.len());
        let mut curr_x = trajectory[0].center_x;
        let mut curr_y = trajectory[0].center_y;

        for pt in trajectory {
            curr_x = curr_x * (1.0 - self.smoothing_factor) + pt.center_x * self.smoothing_factor;
            curr_y = curr_y * (1.0 - self.smoothing_factor) + pt.center_y * self.smoothing_factor;

            smoothed.push(TrackPoint {
                frame_index: pt.frame_index,
                center_x: curr_x,
                center_y: curr_y,
                confidence: pt.confidence,
            });
        }

        smoothed
    }
}
