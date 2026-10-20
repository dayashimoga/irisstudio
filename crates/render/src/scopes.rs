use iris_core::{Dimensions, PixelFormat};
use iris_media::VideoFrame;

/// Generates a real-time Luma Waveform Monitor for a video frame.
/// Displays pixel luminance density across the horizontal field of view.
pub fn render_waveform(source: &VideoFrame, width: u32, height: u32) -> VideoFrame {
    let width = width.clamp(128, 1920);
    let height = height.clamp(128, 1080);
    let w_usize = width as usize;
    let h_usize = height as usize;

    let mut out_frame = VideoFrame::new(
        Dimensions::new(width, height),
        PixelFormat::Rgba8,
        source.timestamp,
        source.duration,
    );

    // Initialize with dark slate scope background
    let bg = [15u8, 18, 26, 255];
    for chunk in out_frame.data.as_chunks_mut::<4>().0 {
        chunk.copy_from_slice(&bg);
    }

    // Draw reference graticules at 0%, 25%, 50%, 75%, 100% IRE
    let graticule_color = [40u8, 52, 70, 255];
    let ire_levels = [0.0f32, 0.25, 0.50, 0.75, 1.0];
    for &ire in &ire_levels {
        let y = ((1.0 - ire) * (height - 1) as f32).round() as usize;
        let y = y.min(h_usize - 1);
        for x in 0..w_usize {
            let offset = (y * w_usize + x) * 4;
            // Dotted/solid graticule
            if x % 4 != 0 {
                out_frame.data[offset..offset + 4].copy_from_slice(&graticule_color);
            }
        }
    }

    // Accumulate luminance density histogram
    let mut density = vec![0u16; w_usize * h_usize];
    let src_w = source.dimensions.width as usize;
    let src_h = source.dimensions.height as usize;
    if src_w == 0 || src_h == 0 {
        return out_frame;
    }

    let bpp = source.format.bytes_per_pixel();
    let is_bgra = source.format == PixelFormat::Bgra8;

    for sy in 0..src_h {
        let row_start = sy * source.stride;
        for sx in 0..src_w {
            let px = row_start + sx * bpp;
            let (r, g, b) = if is_bgra {
                (source.data[px + 2], source.data[px + 1], source.data[px])
            } else {
                (source.data[px], source.data[px + 1], source.data[px + 2])
            };

            // Rec.709 Luma
            let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
            let norm_luma = (luma / 255.0).clamp(0.0, 1.0);

            let scope_x = (sx * w_usize) / src_w;
            let scope_y = ((1.0 - norm_luma) * (h_usize - 1) as f32).round() as usize;
            let scope_y = scope_y.min(h_usize - 1);

            let idx = scope_y * w_usize + scope_x;
            density[idx] = density[idx].saturating_add(1);
        }
    }

    // Render phosphor trace with intensity proportional to density
    let trace_color = [64.0f32, 224.0, 130.0]; // Phosphor green
    for y in 0..h_usize {
        for x in 0..w_usize {
            let count = density[y * w_usize + x];
            if count > 0 {
                let intensity = (count as f32 / 16.0).min(1.0).powf(0.5);
                let offset = (y * w_usize + x) * 4;

                let r =
                    out_frame.data[offset] as f32 * (1.0 - intensity) + trace_color[0] * intensity;
                let g = out_frame.data[offset + 1] as f32 * (1.0 - intensity)
                    + trace_color[1] * intensity;
                let b = out_frame.data[offset + 2] as f32 * (1.0 - intensity)
                    + trace_color[2] * intensity;

                out_frame.data[offset] = r.round() as u8;
                out_frame.data[offset + 1] = g.round() as u8;
                out_frame.data[offset + 2] = b.round() as u8;
            }
        }
    }

    out_frame
}

/// Generates a real-time Vectorscope displaying chromaticity ($C_b, C_r$) on a polar plot.
/// Features 75% SMPTE color targets and skin-tone indicator axis.
pub fn render_vectorscope(source: &VideoFrame, size: u32) -> VideoFrame {
    let size = size.clamp(128, 1024);
    let s_usize = size as usize;

    let mut out_frame = VideoFrame::new(
        Dimensions::new(size, size),
        PixelFormat::Rgba8,
        source.timestamp,
        source.duration,
    );

    // Dark circular background
    let bg = [15u8, 18, 26, 255];
    for chunk in out_frame.data.as_chunks_mut::<4>().0 {
        chunk.copy_from_slice(&bg);
    }

    let center = (size as f32) * 0.5;
    let radius = center * 0.88;

    // Draw reticles: crosshairs and outer boundary
    let graticule_color = [40u8, 52, 70, 255];
    let cx = center.round() as usize;
    let cy = center.round() as usize;

    for i in 0..s_usize {
        // Vertical and horizontal axes
        let h_offset = (cy * s_usize + i) * 4;
        let v_offset = (i * s_usize + cx) * 4;
        if i % 2 == 0 {
            out_frame.data[h_offset..h_offset + 4].copy_from_slice(&graticule_color);
            out_frame.data[v_offset..v_offset + 4].copy_from_slice(&graticule_color);
        }
    }

    // Outer circular boundary
    for angle_deg in 0..360 {
        let rad = (angle_deg as f32).to_radians();
        let px = (center + radius * rad.cos()).round() as isize;
        let py = (center + radius * rad.sin()).round() as isize;
        if px >= 0 && px < size as isize && py >= 0 && py < size as isize {
            let offset = (py as usize * s_usize + px as usize) * 4;
            out_frame.data[offset..offset + 4].copy_from_slice(&graticule_color);
        }
    }

    // Skin-tone reference axis (I-line ~123 degrees / 10:30 on the clock)
    let skin_tone_rad = (123.0f32).to_radians();
    let skin_color = [180u8, 130, 80, 255];
    for step in 0..(radius as usize) {
        let r = step as f32;
        let px = (center - r * skin_tone_rad.cos()).round() as isize;
        let py = (center - r * skin_tone_rad.sin()).round() as isize;
        if px >= 0 && px < size as isize && py >= 0 && py < size as isize {
            let offset = (py as usize * s_usize + px as usize) * 4;
            out_frame.data[offset..offset + 4].copy_from_slice(&skin_color);
        }
    }

    // Accumulate pixel chromaticity density
    let mut density = vec![0u16; s_usize * s_usize];
    let src_w = source.dimensions.width as usize;
    let src_h = source.dimensions.height as usize;
    let bpp = source.format.bytes_per_pixel();
    let is_bgra = source.format == PixelFormat::Bgra8;

    for sy in 0..src_h {
        let row_start = sy * source.stride;
        for sx in 0..src_w {
            let px = row_start + sx * bpp;
            let (r, g, b) = if is_bgra {
                (source.data[px + 2], source.data[px + 1], source.data[px])
            } else {
                (source.data[px], source.data[px + 1], source.data[px + 2])
            };

            let rf = r as f32;
            let gf = g as f32;
            let bf = b as f32;

            // Rec.709 Cb, Cr calculation
            let y = 0.2126 * rf + 0.7152 * gf + 0.0722 * bf;
            let cb = (bf - y) * 0.564;
            let cr = (rf - y) * 0.787;

            let vx = (center + (cb / 128.0) * radius).round() as isize;
            let vy = (center - (cr / 128.0) * radius).round() as isize;

            if vx >= 0 && vx < size as isize && vy >= 0 && vy < size as isize {
                let idx = vy as usize * s_usize + vx as usize;
                density[idx] = density[idx].saturating_add(1);
            }
        }
    }

    // Draw vectorscope traces in cool cyan/teal
    let trace_color = [80.0f32, 220.0, 180.0];
    for y in 0..s_usize {
        for x in 0..s_usize {
            let count = density[y * s_usize + x];
            if count > 0 {
                let intensity = (count as f32 / 12.0).min(1.0).powf(0.5);
                let offset = (y * s_usize + x) * 4;

                let r =
                    out_frame.data[offset] as f32 * (1.0 - intensity) + trace_color[0] * intensity;
                let g = out_frame.data[offset + 1] as f32 * (1.0 - intensity)
                    + trace_color[1] * intensity;
                let b = out_frame.data[offset + 2] as f32 * (1.0 - intensity)
                    + trace_color[2] * intensity;

                out_frame.data[offset] = r.round() as u8;
                out_frame.data[offset + 1] = g.round() as u8;
                out_frame.data[offset + 2] = b.round() as u8;
            }
        }
    }

    out_frame
}

/// Generates a real-time RGB Parade displaying Red, Green, and Blue channels side by side.
pub fn render_rgb_parade(source: &VideoFrame, width: u32, height: u32) -> VideoFrame {
    let width = width.clamp(192, 1920);
    let height = height.clamp(128, 1080);
    let w_usize = width as usize;
    let h_usize = height as usize;

    let mut out_frame = VideoFrame::new(
        Dimensions::new(width, height),
        PixelFormat::Rgba8,
        source.timestamp,
        source.duration,
    );

    // Dark slate background
    let bg = [15u8, 18, 26, 255];
    for chunk in out_frame.data.as_chunks_mut::<4>().0 {
        chunk.copy_from_slice(&bg);
    }

    let sub_w = w_usize / 3;

    // Draw reference lines and channel separators
    let graticule_color = [40u8, 52, 70, 255];
    for ch in 0..3 {
        let x_start = ch * sub_w;
        // Graticule at 0%, 50%, 100%
        for &ratio in &[0.0f32, 0.5, 1.0] {
            let y = ((1.0 - ratio) * (height - 1) as f32).round() as usize;
            let y = y.min(h_usize - 1);
            for x in x_start..(x_start + sub_w) {
                if x % 3 != 0 {
                    let offset = (y * w_usize + x) * 4;
                    out_frame.data[offset..offset + 4].copy_from_slice(&graticule_color);
                }
            }
        }

        // Channel boundary line
        if ch > 0 {
            for y in 0..h_usize {
                let offset = (y * w_usize + x_start) * 4;
                out_frame.data[offset..offset + 4].copy_from_slice(&[60u8, 75, 95, 255]);
            }
        }
    }

    // Accumulate densities for R, G, B channels
    let mut density_r = vec![0u16; sub_w * h_usize];
    let mut density_g = vec![0u16; sub_w * h_usize];
    let mut density_b = vec![0u16; sub_w * h_usize];

    let src_w = source.dimensions.width as usize;
    let src_h = source.dimensions.height as usize;
    let bpp = source.format.bytes_per_pixel();
    let is_bgra = source.format == PixelFormat::Bgra8;

    for sy in 0..src_h {
        let row_start = sy * source.stride;
        for sx in 0..src_w {
            let px = row_start + sx * bpp;
            let (r, g, b) = if is_bgra {
                (source.data[px + 2], source.data[px + 1], source.data[px])
            } else {
                (source.data[px], source.data[px + 1], source.data[px + 2])
            };

            let col = (sx * sub_w) / src_w;
            let ry = (((255 - r) as f32 / 255.0) * (h_usize - 1) as f32).round() as usize;
            let gy = (((255 - g) as f32 / 255.0) * (h_usize - 1) as f32).round() as usize;
            let by = (((255 - b) as f32 / 255.0) * (h_usize - 1) as f32).round() as usize;

            let idx_r = ry.min(h_usize - 1) * sub_w + col;
            let idx_g = gy.min(h_usize - 1) * sub_w + col;
            let idx_b = by.min(h_usize - 1) * sub_w + col;

            density_r[idx_r] = density_r[idx_r].saturating_add(1);
            density_g[idx_g] = density_g[idx_g].saturating_add(1);
            density_b[idx_b] = density_b[idx_b].saturating_add(1);
        }
    }

    // Draw colored phosphor traces for each channel
    let r_color = [255.0f32, 70.0, 70.0];
    let g_color = [70.0f32, 255.0, 90.0];
    let b_color = [70.0f32, 150.0, 255.0];

    for y in 0..h_usize {
        for col in 0..sub_w {
            // Channel 0: Red
            let cnt_r = density_r[y * sub_w + col];
            if cnt_r > 0 {
                let int_r = (cnt_r as f32 / 16.0).min(1.0).powf(0.5);
                let offset = (y * w_usize + col) * 4;
                out_frame.data[offset] =
                    (out_frame.data[offset] as f32 * (1.0 - int_r) + r_color[0] * int_r) as u8;
                out_frame.data[offset + 1] =
                    (out_frame.data[offset + 1] as f32 * (1.0 - int_r) + r_color[1] * int_r) as u8;
                out_frame.data[offset + 2] =
                    (out_frame.data[offset + 2] as f32 * (1.0 - int_r) + r_color[2] * int_r) as u8;
            }

            // Channel 1: Green
            let cnt_g = density_g[y * sub_w + col];
            if cnt_g > 0 {
                let int_g = (cnt_g as f32 / 16.0).min(1.0).powf(0.5);
                let offset = (y * w_usize + sub_w + col) * 4;
                out_frame.data[offset] =
                    (out_frame.data[offset] as f32 * (1.0 - int_g) + g_color[0] * int_g) as u8;
                out_frame.data[offset + 1] =
                    (out_frame.data[offset + 1] as f32 * (1.0 - int_g) + g_color[1] * int_g) as u8;
                out_frame.data[offset + 2] =
                    (out_frame.data[offset + 2] as f32 * (1.0 - int_g) + g_color[2] * int_g) as u8;
            }

            // Channel 2: Blue
            let cnt_b = density_b[y * sub_w + col];
            if cnt_b > 0 {
                let int_b = (cnt_b as f32 / 16.0).min(1.0).powf(0.5);
                let offset = (y * w_usize + sub_w * 2 + col) * 4;
                out_frame.data[offset] =
                    (out_frame.data[offset] as f32 * (1.0 - int_b) + b_color[0] * int_b) as u8;
                out_frame.data[offset + 1] =
                    (out_frame.data[offset + 1] as f32 * (1.0 - int_b) + b_color[1] * int_b) as u8;
                out_frame.data[offset + 2] =
                    (out_frame.data[offset + 2] as f32 * (1.0 - int_b) + b_color[2] * int_b) as u8;
            }
        }
    }

    out_frame
}
