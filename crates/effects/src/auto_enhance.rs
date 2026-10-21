use crate::adjustment::AdjustmentParameters;

/// Content-aware automatic image analysis proposing balanced, conservative adjustments.
pub fn calculate_auto_enhance(pixels: &[u8], width: u32, height: u32) -> AdjustmentParameters {
    if pixels.len() < 4 || width == 0 || height == 0 {
        return AdjustmentParameters::default();
    }

    let total_pixels = (width as usize) * (height as usize);
    // Bounded sample count for fast deterministic execution (<2ms even on 4K/8K images)
    let step = (total_pixels / 10_000).max(1);

    let mut r_sum = 0.0f64;
    let mut g_sum = 0.0f64;
    let mut b_sum = 0.0f64;
    let mut luma_samples = Vec::with_capacity(10_500);

    let mut i = 0;
    while i < total_pixels {
        let offset = i * 4;
        if offset + 3 < pixels.len() {
            let r = pixels[offset] as f64 / 255.0;
            let g = pixels[offset + 1] as f64 / 255.0;
            let b = pixels[offset + 2] as f64 / 255.0;

            r_sum += r;
            g_sum += g;
            b_sum += b;

            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            luma_samples.push(luma);
        }
        i += step;
    }

    let n = luma_samples.len().max(1) as f64;
    let avg_r = r_sum / n;
    let avg_g = g_sum / n;
    let avg_b = b_sum / n;
    let avg_luma = (0.299 * avg_r + 0.587 * avg_g + 0.114 * avg_b).clamp(0.0, 1.0);

    // Compute percentiles for shadow floor and highlight ceiling
    luma_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p05_idx = ((luma_samples.len() as f64) * 0.01).floor() as usize;
    let p95_idx = ((luma_samples.len() as f64) * 0.99).floor() as usize;
    let shadow_floor = luma_samples.get(p05_idx).copied().unwrap_or(0.0);
    let highlight_ceil = luma_samples
        .get(p95_idx.min(luma_samples.len() - 1))
        .copied()
        .unwrap_or(1.0);

    // 1. Exposure adjustment: Target middle gray ~ 0.45
    let exposure = if avg_luma < 0.40 {
        ((0.45 - avg_luma) * 2.5).clamp(0.0, 1.6) as f32
    } else if avg_luma > 0.65 {
        ((0.55 - avg_luma) * 2.0).clamp(-1.4, 0.0) as f32
    } else {
        0.0f32
    };

    // 2. Contrast adjustment: If dynamic range is compressed, expand it gently
    let dynamic_range = highlight_ceil - shadow_floor;
    let contrast = if dynamic_range < 0.6 {
        ((0.7 - dynamic_range) * 40.0).clamp(5.0, 25.0) as f32
    } else if dynamic_range > 0.95 {
        -5.0f32
    } else {
        10.0f32
    };

    // 3. Highlight recovery & Shadow boost
    let highlights = if highlight_ceil > 0.88 || avg_luma > 0.65 {
        -25.0f32
    } else if highlight_ceil < 0.75 {
        10.0f32
    } else {
        0.0f32
    };

    let shadows = if shadow_floor < 0.08 {
        25.0f32
    } else if shadow_floor > 0.25 {
        -10.0f32
    } else {
        10.0f32
    };

    // 4. Conservative Gray-World White Balance adaptation
    let avg_chroma = (avg_r + avg_g + avg_b) / 3.0;
    let temp_delta = (avg_r - avg_b) * 100.0;
    // Counteract harsh color temperature casts conservatively (max +/- 12)
    let temperature = (-temp_delta * 0.35).clamp(-12.0, 12.0) as f32;

    let tint_delta = (avg_g - avg_chroma) * 100.0;
    let tint = (-tint_delta * 0.35).clamp(-10.0, 10.0) as f32;

    // 5. Gentle Vibrance lift for lively results
    let vibrance = 15.0f32;
    let saturation = 5.0f32;

    AdjustmentParameters {
        exposure,
        contrast,
        temperature,
        tint,
        highlights,
        shadows,
        saturation,
        vibrance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_enhance_dark_image() {
        // Create 100x100 dark image (luminance ~ 0.1)
        let pixels = vec![30u8; 100 * 100 * 4];
        let adj = calculate_auto_enhance(&pixels, 100, 100);
        assert!(adj.exposure > 0.5, "Exposure should boost dark image");
        assert!(adj.shadows > 0.0, "Shadows should be lifted");
    }

    #[test]
    fn test_auto_enhance_bright_image() {
        // Create 100x100 bright image (luminance ~ 0.9)
        let pixels = vec![230u8; 100 * 100 * 4];
        let adj = calculate_auto_enhance(&pixels, 100, 100);
        assert!(
            adj.exposure < 0.0,
            "Exposure should decrease for overly bright image"
        );
        assert!(adj.highlights < 0.0, "Highlights should be recovered");
    }
}
