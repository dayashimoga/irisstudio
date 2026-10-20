use iris_effects::{
    apply_adjustments, apply_gaussian_blur, apply_vignette, blend_pixel, AdjustmentParameters,
    BlendMode, CurveLut, SplinePoint,
};

#[test]
fn test_exposure_and_contrast_adjustments() {
    let mut pixels = vec![128, 128, 128, 255]; // Neutral mid-grey

    let params = AdjustmentParameters {
        exposure: 1.0,
        ..Default::default()
    };

    apply_adjustments(&mut pixels, &params);

    // After +1.0 EV exposure, brightness must increase significantly
    assert!(pixels[0] > 128);
    assert!(pixels[1] > 128);
    assert!(pixels[2] > 128);
    assert_eq!(pixels[3], 255);
}

#[test]
fn test_layer_blend_modes() {
    let white = [255, 255, 255, 255];
    let black = [0, 0, 0, 255];
    let grey = [128, 128, 128, 255];

    // Multiply with white is identity
    let mul_white = blend_pixel(grey, white, BlendMode::Multiply, 1.0);
    assert_eq!(mul_white[0], 128);

    // Multiply with black is black
    let mul_black = blend_pixel(grey, black, BlendMode::Multiply, 1.0);
    assert_eq!(mul_black[0], 0);

    // Screen with black is identity
    let screen_black = blend_pixel(grey, black, BlendMode::Screen, 1.0);
    assert_eq!(screen_black[0], 128);
}

#[test]
fn test_curve_lut_generation() {
    let pts = vec![
        SplinePoint { x: 0.0, y: 0.0 },
        SplinePoint { x: 0.5, y: 0.8 }, // Brightened midtones
        SplinePoint { x: 1.0, y: 1.0 },
    ];

    let lut = CurveLut::from_points(pts);
    assert_eq!(lut.map(0), 0);
    assert_eq!(lut.map(255), 255);
    // Midpoint (128) should map higher than 128
    assert!(lut.map(128) > 128);
}

#[test]
fn test_filters_execution() {
    let width = 64;
    let height = 64;
    let mut pixels = vec![128u8; width * height * 4];

    // Test vignette
    apply_vignette(&mut pixels, width as u32, height as u32, 50.0, 0.5, 0.5);

    // Center pixel should be unchanged
    let center_idx = (32 * 64 + 32) * 4;
    assert_eq!(pixels[center_idx], 128);

    // Corner pixel should be darkened
    let corner_idx = 0;
    assert!(pixels[corner_idx] < 128);

    // Test blur
    apply_gaussian_blur(&mut pixels, width as u32, height as u32, 2.0);
}

#[test]
fn test_levels_adjustment() {
    let mut pixels = vec![10u8, 128u8, 240u8, 255u8];

    // Darken black point to 20, brighten white point to 230
    let params = iris_effects::LevelsParameters {
        in_black: 20.0,
        in_gamma: 1.0,
        in_white: 230.0,
        out_black: 0.0,
        out_white: 255.0,
    };

    iris_effects::apply_levels(&mut pixels, &params);

    // Value 10 is below in_black (20), so it should clamp to 0
    assert_eq!(pixels[0], 0);
    // Value 240 is above in_white (230), so it should clamp to 255
    assert_eq!(pixels[2], 255);
}

#[test]
fn test_edge_preserving_denoise() {
    let width = 32;
    let height = 32;
    // Create an image with a flat noisy region and a sharp step edge
    let mut pixels = vec![100u8; width * height * 4];
    // Add artificial noise to left half
    pixels[0] = 120; // noisy spike
                     // Right half has bright sharp step
    for y in 0..height {
        for x in 16..width {
            let off = (y * width + x) * 4;
            pixels[off] = 220;
            pixels[off + 1] = 220;
            pixels[off + 2] = 220;
        }
    }

    iris_effects::apply_denoise(&mut pixels, width as u32, height as u32, 2, 40.0);

    // The sharp edge at x=20 should remain bright
    let edge_sample = (16 * width + 20) * 4;
    assert!(pixels[edge_sample] > 200);
}

#[test]
fn test_retouch_red_eye_removal() {
    let width = 32;
    let height = 32;
    let mut pixels = vec![120u8; width * height * 4];

    // Simulate intense flash red-eye reflection at center (16, 16)
    let center_idx = (16 * width + 16) * 4;
    pixels[center_idx] = 240; // High red
    pixels[center_idx + 1] = 30; // Low green
    pixels[center_idx + 2] = 30; // Low blue

    iris_effects::apply_red_eye_removal(&mut pixels, width as u32, height as u32, 16, 16, 6.0, 1.5);

    // Red channel at pupil should be significantly reduced and neutralized
    assert!(pixels[center_idx] < 100);
}

#[test]
fn test_retouch_clone_stamp_and_heal() {
    let width = 32;
    let height = 32;
    let mut pixels = vec![50u8; width * height * 4];

    // Source area has chromatic texture
    let src_idx = (8 * width + 8) * 4;
    pixels[src_idx] = 240;
    pixels[src_idx + 1] = 60;
    pixels[src_idx + 2] = 60;

    // Clone from (8, 8) to (24, 24)
    iris_effects::apply_clone_stamp(
        &mut pixels,
        width as u32,
        height as u32,
        8,
        8,
        24,
        24,
        4.0,
        1.0,
        1.0,
    );

    let dst_idx = (24 * width + 24) * 4;
    assert_eq!(pixels[dst_idx], 240);

    // Heal brush
    iris_effects::apply_heal_brush(
        &mut pixels,
        width as u32,
        height as u32,
        8,
        8,
        20,
        20,
        3.0,
        0.8,
    );
    let heal_idx = (20 * width + 20) * 4;
    assert!(pixels[heal_idx] > 50);
}

#[test]
fn test_photo_layers_composition() {
    let canvas_w = 40;
    let canvas_h = 40;

    // Bottom layer: blue background
    let mut layer1_pixels = vec![0u8; 40 * 40 * 4];
    for chunk in layer1_pixels.as_chunks_mut::<4>().0 {
        chunk[0] = 0;
        chunk[1] = 0;
        chunk[2] = 255;
        chunk[3] = 255;
    }
    let layer1 = iris_effects::PhotoLayer::new("l1", "Background", 40, 40, layer1_pixels);

    // Top layer: semi-transparent red rectangle with screen blend mode
    let mut layer2_pixels = vec![0u8; 20 * 20 * 4];
    for chunk in layer2_pixels.as_chunks_mut::<4>().0 {
        chunk[0] = 255;
        chunk[1] = 0;
        chunk[2] = 0;
        chunk[3] = 255;
    }
    let mut layer2 = iris_effects::PhotoLayer::new("l2", "Overlay", 20, 20, layer2_pixels);
    layer2.opacity = 0.8;
    layer2.blend_mode = iris_effects::BlendMode::Screen;
    layer2.offset_x = 10;
    layer2.offset_y = 10;

    let composed = iris_effects::compose_photo_layers(canvas_w, canvas_h, &[layer1, layer2]);
    assert_eq!(composed.len(), 40 * 40 * 4);

    // Overlap center at (15, 15) should blend red screen on blue -> magenta tint
    let center_idx = (15 * 40 + 15) * 4;
    assert!(composed[center_idx] > 100); // Red channel
    assert!(composed[center_idx + 2] > 100); // Blue channel
}
