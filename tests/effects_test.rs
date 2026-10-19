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
