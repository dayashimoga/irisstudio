use rayon::prelude::*;

pub fn apply_vignette(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    amount: f32, // -100.0 to +100.0 (negative = white vignette, positive = dark vignette)
    midpoint: f32, // 0.0 to 1.0 (default 0.5)
    feather: f32, // 0.0 to 1.0 (default 0.5)
) {
    if amount == 0.0 {
        return;
    }

    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_radius = (cx * cx + cy * cy).sqrt();

    let inner_radius = (max_radius * (midpoint.clamp(0.05, 0.95) - feather * 0.4)).max(0.0);
    let outer_radius =
        (max_radius * (midpoint.clamp(0.05, 0.95) + feather * 0.4)).max(inner_radius + 1.0);
    let strength = amount / 100.0;

    pixels
        .par_chunks_mut(width as usize * 4)
        .enumerate()
        .for_each(|(y, row)| {
            let dy = y as f32 - cy;
            for x in 0..width as usize {
                let dx = x as f32 - cx;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > inner_radius {
                    let factor =
                        ((dist - inner_radius) / (outer_radius - inner_radius)).clamp(0.0, 1.0);
                    let smooth_factor = factor * factor * (3.0 - 2.0 * factor);

                    let darken = 1.0 - smooth_factor * strength;
                    let offset = x * 4;

                    row[offset] = ((row[offset] as f32 * darken).round()).clamp(0.0, 255.0) as u8;
                    row[offset + 1] =
                        ((row[offset + 1] as f32 * darken).round()).clamp(0.0, 255.0) as u8;
                    row[offset + 2] =
                        ((row[offset + 2] as f32 * darken).round()).clamp(0.0, 255.0) as u8;
                }
            }
        });
}

pub fn apply_unsharp_mask(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    amount: f32, // 0.0 to 100.0
    radius: f32, // blur radius for mask
) {
    if amount <= 0.0 {
        return;
    }

    // Clone original for blur pass
    let mut blurred = pixels.to_vec();
    apply_gaussian_blur(&mut blurred, width, height, radius);

    let strength = amount / 50.0; // scale factor

    pixels
        .par_chunks_mut(4)
        .zip(blurred.par_chunks(4))
        .for_each(|(orig, blur)| {
            for c in 0..3 {
                let o = orig[c] as f32;
                let b = blur[c] as f32;
                let diff = o - b;
                let sharp = o + diff * strength;
                orig[c] = sharp.round().clamp(0.0, 255.0) as u8;
            }
        });
}

/// Fast separable Gaussian-approximation blur
pub fn apply_gaussian_blur(pixels: &mut [u8], width: u32, height: u32, radius: f32) {
    let r = radius.round() as i32;
    if r <= 0 {
        return;
    }

    let w = width as usize;
    let h = height as usize;
    let mut temp = vec![0u8; pixels.len()];

    // Horizontal pass
    temp.par_chunks_mut(w * 4)
        .enumerate()
        .for_each(|(y, dst_row)| {
            let src_row = &pixels[y * w * 4..(y + 1) * w * 4];
            for x in 0..w {
                let mut sum_r = 0f32;
                let mut sum_g = 0f32;
                let mut sum_b = 0f32;
                let mut count = 0f32;

                for k in -r..=r {
                    let sample_x = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                    let weight = 1.0 - (k.abs() as f32 / (r + 1) as f32);
                    let off = sample_x * 4;
                    sum_r += src_row[off] as f32 * weight;
                    sum_g += src_row[off + 1] as f32 * weight;
                    sum_b += src_row[off + 2] as f32 * weight;
                    count += weight;
                }

                let dst_off = x * 4;
                dst_row[dst_off] = (sum_r / count).round().clamp(0.0, 255.0) as u8;
                dst_row[dst_off + 1] = (sum_g / count).round().clamp(0.0, 255.0) as u8;
                dst_row[dst_off + 2] = (sum_b / count).round().clamp(0.0, 255.0) as u8;
                dst_row[dst_off + 3] = src_row[dst_off + 3];
            }
        });

    // Vertical pass
    pixels
        .par_chunks_mut(4)
        .enumerate()
        .for_each(|(idx, dst_px)| {
            let x = idx % w;
            let y = idx / w;

            let mut sum_r = 0f32;
            let mut sum_g = 0f32;
            let mut sum_b = 0f32;
            let mut count = 0f32;

            for k in -r..=r {
                let sample_y = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                let weight = 1.0 - (k.abs() as f32 / (r + 1) as f32);
                let off = (sample_y * w + x) * 4;
                sum_r += temp[off] as f32 * weight;
                sum_g += temp[off + 1] as f32 * weight;
                sum_b += temp[off + 2] as f32 * weight;
                count += weight;
            }

            dst_px[0] = (sum_r / count).round().clamp(0.0, 255.0) as u8;
            dst_px[1] = (sum_g / count).round().clamp(0.0, 255.0) as u8;
            dst_px[2] = (sum_b / count).round().clamp(0.0, 255.0) as u8;
            dst_px[3] = temp[(y * w + x) * 4 + 3];
        });
}
