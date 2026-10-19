//! Retouching tools: Red-eye correction, Clone stamp, Heal brush, and Freehand painting/eraser.

/// Detects abnormal red-eye reflection in a localized circular region and desaturates it.
pub fn apply_red_eye_removal(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
    radius: f32,
    threshold: f32, // typically 1.4 to 2.0 (ratio of red to green/blue mean)
) {
    if radius <= 0.0 {
        return;
    }

    let r_i = radius.ceil() as i32;
    let min_x = (center_x as i32 - r_i).max(0) as u32;
    let max_x = (center_x as i32 + r_i).min(width as i32 - 1) as u32;
    let min_y = (center_y as i32 - r_i).max(0) as u32;
    let max_y = (center_y as i32 + r_i).min(height as i32 - 1) as u32;

    for y in min_y..=max_y {
        let dy = y as f32 - center_y as f32;
        for x in min_x..=max_x {
            let dx = x as f32 - center_x as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                let off = ((y * width + x) * 4) as usize;
                let r = pixels[off] as f32;
                let g = pixels[off + 1] as f32;
                let b = pixels[off + 2] as f32;

                let gb_avg = (g + b) / 2.0;
                let ratio = if gb_avg > 0.0 { r / gb_avg } else { 1.0 };

                if ratio > threshold && r > 40.0 {
                    let t = 1.0 - (dist / radius);
                    let falloff = t * t * (3.0 - 2.0 * t);

                    // Pupil target is dark neutral grey/brown matching ambient eye tone
                    let target_r = gb_avg * 0.7;
                    let target_g = g * 0.85;
                    let target_b = b * 0.85;

                    pixels[off] = (r * (1.0 - falloff) + target_r * falloff)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    pixels[off + 1] = (g * (1.0 - falloff) + target_g * falloff)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    pixels[off + 2] = (b * (1.0 - falloff) + target_b * falloff)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

/// Copies texture from source point (src_x, src_y) to destination point (dst_x, dst_y).
/// hardness: 0.0 to 1.0 (0.0 = soft edge, 1.0 = hard edge)
/// opacity: 0.0 to 1.0
#[allow(clippy::too_many_arguments)]
pub fn apply_clone_stamp(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    radius: f32,
    hardness: f32,
    opacity: f32,
) {
    if radius <= 0.0 || opacity <= 0.0 {
        return;
    }

    let r_i = radius.ceil() as i32;
    let inner_radius = radius * hardness.clamp(0.0, 1.0);
    let orig = pixels.to_vec();

    for dy in -r_i..=r_i {
        let cur_dst_y = dst_y + dy;
        let cur_src_y = src_y + dy;

        if cur_dst_y < 0
            || cur_dst_y >= height as i32
            || cur_src_y < 0
            || cur_src_y >= height as i32
        {
            continue;
        }

        for dx in -r_i..=r_i {
            let cur_dst_x = dst_x + dx;
            let cur_src_x = src_x + dx;

            if cur_dst_x < 0
                || cur_dst_x >= width as i32
                || cur_src_x < 0
                || cur_src_x >= width as i32
            {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= radius {
                let weight = if dist <= inner_radius {
                    1.0
                } else {
                    let t = (radius - dist) / (radius - inner_radius).max(0.001);
                    t * t * (3.0 - 2.0 * t)
                } * opacity.clamp(0.0, 1.0);

                let src_off = ((cur_src_y as u32 * width + cur_src_x as u32) * 4) as usize;
                let dst_off = ((cur_dst_y as u32 * width + cur_dst_x as u32) * 4) as usize;

                for c in 0..4 {
                    let s = orig[src_off + c] as f32;
                    let d = pixels[dst_off + c] as f32;
                    pixels[dst_off + c] =
                        (d * (1.0 - weight) + s * weight).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

/// Heal brush: transfers source high-frequency details while harmonizing luminance with destination.
#[allow(clippy::too_many_arguments)]
pub fn apply_heal_brush(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    radius: f32,
    hardness: f32,
) {
    if radius <= 0.0 {
        return;
    }

    let r_i = radius.ceil() as i32;
    let inner_radius = radius * hardness.clamp(0.0, 1.0);
    let orig = pixels.to_vec();

    for dy in -r_i..=r_i {
        let cur_dst_y = dst_y + dy;
        let cur_src_y = src_y + dy;

        if cur_dst_y < 0
            || cur_dst_y >= height as i32
            || cur_src_y < 0
            || cur_src_y >= height as i32
        {
            continue;
        }

        for dx in -r_i..=r_i {
            let cur_dst_x = dst_x + dx;
            let cur_src_x = src_x + dx;

            if cur_dst_x < 0
                || cur_dst_x >= width as i32
                || cur_src_x < 0
                || cur_src_x >= width as i32
            {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= radius {
                let weight = if dist <= inner_radius {
                    1.0
                } else {
                    let t = (radius - dist) / (radius - inner_radius).max(0.001);
                    t * t * (3.0 - 2.0 * t)
                };

                let src_off = ((cur_src_y as u32 * width + cur_src_x as u32) * 4) as usize;
                let dst_off = ((cur_dst_y as u32 * width + cur_dst_x as u32) * 4) as usize;

                let s_r = orig[src_off] as f32;
                let s_g = orig[src_off + 1] as f32;
                let s_b = orig[src_off + 2] as f32;

                let d_r = orig[dst_off] as f32;
                let d_g = orig[dst_off + 1] as f32;
                let d_b = orig[dst_off + 2] as f32;

                let l_src = 0.299 * s_r + 0.587 * s_g + 0.114 * s_b;
                let l_dst = 0.299 * d_r + 0.587 * d_g + 0.114 * d_b;

                // High-frequency texture delta from source
                let delta_r = s_r - l_src;
                let delta_g = s_g - l_src;
                let delta_b = s_b - l_src;

                // Ambient lighting from destination combined with source detail
                let healed_r = (l_dst + delta_r).clamp(0.0, 255.0);
                let healed_g = (l_dst + delta_g).clamp(0.0, 255.0);
                let healed_b = (l_dst + delta_b).clamp(0.0, 255.0);

                pixels[dst_off] = (d_r * (1.0 - weight) + healed_r * weight)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                pixels[dst_off + 1] = (d_g * (1.0 - weight) + healed_g * weight)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                pixels[dst_off + 2] = (d_b * (1.0 - weight) + healed_b * weight)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Applies freehand paint brush or eraser along a trajectory of points.
#[allow(clippy::too_many_arguments)]
pub fn apply_brush_stroke(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    points: &[(f32, f32)],
    color: [u8; 4],
    radius: f32,
    hardness: f32,
    erase: bool,
) {
    if points.is_empty() || radius <= 0.0 {
        return;
    }

    let r_i = radius.ceil() as i32;
    let inner_radius = radius * hardness.clamp(0.0, 1.0);

    for &(pt_x, pt_y) in points {
        let center_x = pt_x.round() as i32;
        let center_y = pt_y.round() as i32;

        for dy in -r_i..=r_i {
            let y = center_y + dy;
            if y < 0 || y >= height as i32 {
                continue;
            }

            for dx in -r_i..=r_i {
                let x = center_x + dx;
                if x < 0 || x >= width as i32 {
                    continue;
                }

                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius {
                    let weight = if dist <= inner_radius {
                        1.0
                    } else {
                        let t = (radius - dist) / (radius - inner_radius).max(0.001);
                        t * t * (3.0 - 2.0 * t)
                    };

                    let off = ((y as u32 * width + x as u32) * 4) as usize;

                    if erase {
                        let cur_a = pixels[off + 3] as f32;
                        pixels[off + 3] = (cur_a * (1.0 - weight)).round().clamp(0.0, 255.0) as u8;
                    } else {
                        let alpha = (color[3] as f32 / 255.0) * weight;
                        for c in 0..3 {
                            let src_c = color[c] as f32;
                            let dst_c = pixels[off + c] as f32;
                            pixels[off + c] = (dst_c * (1.0 - alpha) + src_c * alpha)
                                .round()
                                .clamp(0.0, 255.0)
                                as u8;
                        }
                        pixels[off + 3] = (pixels[off + 3] as f32)
                            .max(color[3] as f32 * weight)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
}
