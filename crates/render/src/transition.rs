//! Video transition rendering algorithms.

use iris_core::{IrisError, IrisResult};
use iris_media::VideoFrame;
use iris_timeline::TransitionKind;
use rayon::prelude::*;

/// Blends two video frames using the specified transition kind and normalized progress t in [0.0, 1.0].
/// Outputs the result into `dst`.
pub fn render_transition(
    frame_a: &VideoFrame,
    frame_b: &VideoFrame,
    kind: TransitionKind,
    progress: f32,
    dst: &mut VideoFrame,
) -> IrisResult<()> {
    if frame_a.dimensions != frame_b.dimensions || frame_a.dimensions != dst.dimensions {
        return Err(IrisError::InvalidParameter(
            "Frame dimensions must match for transition rendering".to_string(),
        ));
    }

    let t = progress.clamp(0.0, 1.0);
    let width = dst.dimensions.width as usize;
    let height = dst.dimensions.height as usize;
    let row_bytes = width * 4;

    match kind {
        TransitionKind::CrossDissolve => {
            let inv_t = 1.0 - t;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .for_each(|((dst_row, a_row), b_row)| {
                    for i in 0..row_bytes {
                        let a = a_row[i] as f32;
                        let b = b_row[i] as f32;
                        dst_row[i] = (a * inv_t + b * t).round() as u8;
                    }
                });
        }
        TransitionKind::FadeToBlack => {
            // 0.0 -> 0.5: fade A down to black
            // 0.5 -> 1.0: fade black up to B
            let (is_a, alpha) = if t < 0.5 {
                (true, (1.0 - t * 2.0).clamp(0.0, 1.0))
            } else {
                (false, ((t - 0.5) * 2.0).clamp(0.0, 1.0))
            };

            let src_data = if is_a { &frame_a.data } else { &frame_b.data };

            dst.data
                .par_chunks_mut(row_bytes)
                .zip(src_data.par_chunks(row_bytes))
                .for_each(|(dst_row, src_row)| {
                    for chunk_idx in 0..width {
                        let idx = chunk_idx * 4;
                        dst_row[idx] = (src_row[idx] as f32 * alpha).round() as u8;
                        dst_row[idx + 1] = (src_row[idx + 1] as f32 * alpha).round() as u8;
                        dst_row[idx + 2] = (src_row[idx + 2] as f32 * alpha).round() as u8;
                        dst_row[idx + 3] = 255;
                    }
                });
        }
        TransitionKind::DipToColor { r, g, b } => {
            // 0.0 -> 0.5: blend A to (r, g, b)
            // 0.5 -> 1.0: blend (r, g, b) to B
            let (is_a, factor) = if t < 0.5 {
                (true, (t * 2.0).clamp(0.0, 1.0))
            } else {
                (false, (1.0 - (t - 0.5) * 2.0).clamp(0.0, 1.0))
            };

            let src_data = if is_a { &frame_a.data } else { &frame_b.data };
            let inv_f = 1.0 - factor;
            let target_r = r as f32 * factor;
            let target_g = g as f32 * factor;
            let target_b = b as f32 * factor;

            dst.data
                .par_chunks_mut(row_bytes)
                .zip(src_data.par_chunks(row_bytes))
                .for_each(|(dst_row, src_row)| {
                    for chunk_idx in 0..width {
                        let idx = chunk_idx * 4;
                        dst_row[idx] = (src_row[idx] as f32 * inv_f + target_r).round() as u8;
                        dst_row[idx + 1] =
                            (src_row[idx + 1] as f32 * inv_f + target_g).round() as u8;
                        dst_row[idx + 2] =
                            (src_row[idx + 2] as f32 * inv_f + target_b).round() as u8;
                        dst_row[idx + 3] = 255;
                    }
                });
        }
        TransitionKind::WipeLeft => {
            // B sweeps from right to left
            let split_x = ((1.0 - t) * width as f32) as usize;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .for_each(|((dst_row, a_row), b_row)| {
                    for x in 0..width {
                        let idx = x * 4;
                        let src = if x >= split_x {
                            &b_row[idx..idx + 4]
                        } else {
                            &a_row[idx..idx + 4]
                        };
                        dst_row[idx..idx + 4].copy_from_slice(src);
                    }
                });
        }
        TransitionKind::WipeRight => {
            // B sweeps from left to right
            let split_x = (t * width as f32) as usize;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .for_each(|((dst_row, a_row), b_row)| {
                    for x in 0..width {
                        let idx = x * 4;
                        let src = if x < split_x {
                            &b_row[idx..idx + 4]
                        } else {
                            &a_row[idx..idx + 4]
                        };
                        dst_row[idx..idx + 4].copy_from_slice(src);
                    }
                });
        }
        TransitionKind::WipeUp => {
            // B sweeps from bottom to top
            let split_y = ((1.0 - t) * height as f32) as usize;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .enumerate()
                .for_each(|(y, ((dst_row, a_row), b_row))| {
                    let src = if y >= split_y { b_row } else { a_row };
                    dst_row.copy_from_slice(src);
                });
        }
        TransitionKind::WipeDown => {
            // B sweeps from top to bottom
            let split_y = (t * height as f32) as usize;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .enumerate()
                .for_each(|(y, ((dst_row, a_row), b_row))| {
                    let src = if y < split_y { b_row } else { a_row };
                    dst_row.copy_from_slice(src);
                });
        }
        TransitionKind::Slide | TransitionKind::PushLeft => {
            // Clip A moves left, Clip B enters from right
            let offset_x = (t * width as f32) as isize;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .for_each(|((dst_row, a_row), b_row)| {
                    for x in 0..width {
                        let dst_idx = x * 4;
                        let ax = x as isize + offset_x;
                        if ax < width as isize {
                            let src_idx = ax as usize * 4;
                            dst_row[dst_idx..dst_idx + 4]
                                .copy_from_slice(&a_row[src_idx..src_idx + 4]);
                        } else {
                            let bx = x as isize - (width as isize - offset_x);
                            if bx >= 0 && (bx as usize) < width {
                                let src_idx = bx as usize * 4;
                                dst_row[dst_idx..dst_idx + 4]
                                    .copy_from_slice(&b_row[src_idx..src_idx + 4]);
                            }
                        }
                    }
                });
        }
        TransitionKind::PushRight => {
            // Clip A moves right, Clip B enters from left
            let offset_x = (t * width as f32) as isize;
            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .for_each(|((dst_row, a_row), b_row)| {
                    for x in 0..width {
                        let dst_idx = x * 4;
                        if (x as isize) < offset_x {
                            let bx = (x as isize) + (width as isize - offset_x);
                            if bx >= 0 && (bx as usize) < width {
                                let src_idx = bx as usize * 4;
                                dst_row[dst_idx..dst_idx + 4]
                                    .copy_from_slice(&b_row[src_idx..src_idx + 4]);
                            }
                        } else {
                            let ax = x as isize - offset_x;
                            if ax >= 0 && (ax as usize) < width {
                                let src_idx = ax as usize * 4;
                                dst_row[dst_idx..dst_idx + 4]
                                    .copy_from_slice(&a_row[src_idx..src_idx + 4]);
                            }
                        }
                    }
                });
        }
        TransitionKind::IrisRound => {
            // Circular aperture reveal expanding from center
            let cx = width as f32 / 2.0;
            let cy = height as f32 / 2.0;
            let max_r = (cx * cx + cy * cy).sqrt();
            let current_r = t * max_r;
            let r_sq = current_r * current_r;

            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .zip(frame_b.data.par_chunks(row_bytes))
                .enumerate()
                .for_each(|(y, ((dst_row, a_row), b_row))| {
                    let dy = y as f32 - cy;
                    let dy_sq = dy * dy;
                    for x in 0..width {
                        let dx = x as f32 - cx;
                        let dist_sq = dx * dx + dy_sq;
                        let idx = x * 4;
                        let src = if dist_sq <= r_sq {
                            &b_row[idx..idx + 4]
                        } else {
                            &a_row[idx..idx + 4]
                        };
                        dst_row[idx..idx + 4].copy_from_slice(src);
                    }
                });
        }
        TransitionKind::ZoomIn => {
            // Clip B scales up from center (0.05 -> 1.0) while dissolving in over A
            let scale = (0.05 + 0.95 * t).max(0.01);
            let cx = width as f32 / 2.0;
            let cy = height as f32 / 2.0;
            let inv_scale = 1.0 / scale;

            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_a.data.par_chunks(row_bytes))
                .enumerate()
                .for_each(|(y, (dst_row, a_row))| {
                    let dy = y as f32 - cy;
                    for x in 0..width {
                        let dx = x as f32 - cx;
                        let bx = dx * inv_scale + cx;
                        let by = dy * inv_scale + cy;
                        let dst_idx = x * 4;

                        if bx >= 0.0 && bx < width as f32 && by >= 0.0 && by < height as f32 {
                            let src_bx = bx.floor() as usize;
                            let src_by = by.floor() as usize;
                            let b_idx = (src_by * width + src_bx) * 4;

                            // Alpha blend B over A using transition progress t
                            for c in 0..3 {
                                let va = a_row[dst_idx + c] as f32;
                                let vb = frame_b.data[b_idx + c] as f32;
                                dst_row[dst_idx + c] = (va * (1.0 - t) + vb * t).round() as u8;
                            }
                            dst_row[dst_idx + 3] = 255;
                        } else {
                            dst_row[dst_idx..dst_idx + 4]
                                .copy_from_slice(&a_row[dst_idx..dst_idx + 4]);
                        }
                    }
                });
        }
        TransitionKind::ZoomOut => {
            // Clip A scales down to center (1.0 -> 0.05) revealing B underneath
            let scale = (1.0 - 0.95 * t).max(0.01);
            let cx = width as f32 / 2.0;
            let cy = height as f32 / 2.0;
            let inv_scale = 1.0 / scale;

            dst.data
                .par_chunks_mut(row_bytes)
                .zip(frame_b.data.par_chunks(row_bytes))
                .enumerate()
                .for_each(|(y, (dst_row, b_row))| {
                    let dy = y as f32 - cy;
                    for x in 0..width {
                        let dx = x as f32 - cx;
                        let ax = dx * inv_scale + cx;
                        let ay = dy * inv_scale + cy;
                        let dst_idx = x * 4;

                        if ax >= 0.0 && ax < width as f32 && ay >= 0.0 && ay < height as f32 {
                            let src_ax = ax.floor() as usize;
                            let src_ay = ay.floor() as usize;
                            let a_idx = (src_ay * width + src_ax) * 4;

                            let a_weight = 1.0 - t;
                            for c in 0..3 {
                                let va = frame_a.data[a_idx + c] as f32;
                                let vb = b_row[dst_idx + c] as f32;
                                dst_row[dst_idx + c] =
                                    (va * a_weight + vb * (1.0 - a_weight)).round() as u8;
                            }
                            dst_row[dst_idx + 3] = 255;
                        } else {
                            dst_row[dst_idx..dst_idx + 4]
                                .copy_from_slice(&b_row[dst_idx..dst_idx + 4]);
                        }
                    }
                });
        }
    }

    Ok(())
}
