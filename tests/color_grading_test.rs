use iris_core::{Dimensions, PixelFormat, RationalTime};
use iris_effects::{CurveLut, Lut3D, LutPreset, RgbCurves, SplinePoint};
use iris_media::VideoFrame;
use iris_render::{render_rgb_parade, render_vectorscope, render_waveform};

#[test]
fn test_lut_3d_identity_and_preset() {
    let identity = Lut3D::identity(17);
    let [r, g, b] = identity.sample_tetrahedral(0.5, 0.5, 0.5);
    assert!((r - 0.5).abs() < 1e-3);
    assert!((g - 0.5).abs() < 1e-3);
    assert!((b - 0.5).abs() < 1e-3);

    // Apply identity to buffer
    let mut pixels = vec![128u8, 64, 200, 255];
    let orig = pixels.clone();
    identity.apply_rgba(&mut pixels, 1.0);
    // Discrepancy should be at most 1 code value due to 8-bit quantization
    assert!((pixels[0] as i16 - orig[0] as i16).abs() <= 1);
    assert!((pixels[1] as i16 - orig[1] as i16).abs() <= 1);
    assert!((pixels[2] as i16 - orig[2] as i16).abs() <= 1);

    // Test Teal & Orange preset
    let teal_orange = Lut3D::preset(LutPreset::TealAndOrange);
    let mut sample_px = vec![180u8, 140, 100, 255]; // Warm skin-tone highlight
    teal_orange.apply_rgba(&mut sample_px, 1.0);
    // Highlights should push red/orange and suppress blue
    assert!(sample_px[0] >= sample_px[2]);
}

#[test]
fn test_lut_3d_cube_parser() {
    let cube_str = r#"
# Simple test 2x2x2 cube LUT
TITLE "TestMini"
LUT_3D_SIZE 2
DOMAIN_MIN 0.0 0.0 0.0
DOMAIN_MAX 1.0 1.0 1.0

0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
"#;

    let lut = Lut3D::parse_cube(cube_str).expect("Valid .cube should parse");
    assert_eq!(lut.title, "TestMini");
    assert_eq!(lut.size, 2);
    assert_eq!(lut.table.len(), 8);

    // Tetrahedral sampling at corner
    let [r, g, b] = lut.sample_tetrahedral(1.0, 0.0, 0.0);
    assert!((r - 1.0).abs() < 1e-3);
    assert!((g - 0.0).abs() < 1e-3);
    assert!((b - 0.0).abs() < 1e-3);
}

#[test]
fn test_monotonic_cubic_spline_curves() {
    // S-curve contrast control points
    let points = vec![
        SplinePoint { x: 0.0, y: 0.0 },
        SplinePoint { x: 0.25, y: 0.15 }, // Dip shadows
        SplinePoint { x: 0.75, y: 0.85 }, // Boost highlights
        SplinePoint { x: 1.0, y: 1.0 },
    ];

    let lut = CurveLut::from_points(points);
    assert_eq!(lut.lut.len(), 256);

    // Check boundary values
    assert_eq!(lut.lut[0], 0);
    assert_eq!(lut.lut[255], 255);

    // Check strict monotonicity (no overshoot dips)
    for i in 1..256 {
        assert!(
            lut.lut[i] >= lut.lut[i - 1],
            "Monotonic spline violated at index {}: {} < {}",
            i,
            lut.lut[i],
            lut.lut[i - 1]
        );
    }

    // Shadow at 25% should be darker than identity (64)
    assert!(lut.lut[64] < 64);
    // Highlight at 75% should be brighter than identity (191)
    assert!(lut.lut[191] > 191);
}

#[test]
fn test_multi_channel_rgb_curves() {
    let master_points = vec![
        SplinePoint { x: 0.0, y: 0.0 },
        SplinePoint { x: 1.0, y: 1.0 },
    ];
    // Red channel boost
    let red_points = vec![
        SplinePoint { x: 0.0, y: 0.0 },
        SplinePoint { x: 0.5, y: 0.75 },
        SplinePoint { x: 1.0, y: 1.0 },
    ];
    let green_points = master_points.clone();
    let blue_points = master_points.clone();

    let rgb_curves = RgbCurves::new(master_points, red_points, green_points, blue_points);

    let mut pixels = vec![128u8, 128, 128, 255];
    rgb_curves.apply_rgba(&mut pixels);

    // Red channel should be boosted above 128, Green and Blue stay at 128
    assert!(pixels[0] > 128);
    assert_eq!(pixels[1], 128);
    assert_eq!(pixels[2], 128);
}

#[test]
fn test_diagnostic_color_scopes() {
    let test_frame = VideoFrame::create_test_pattern(
        Dimensions::new(640, 360),
        RationalTime::new(0, 30),
        RationalTime::new(1, 30),
    );

    // 1. Luma Waveform
    let waveform = render_waveform(&test_frame, 256, 256);
    assert_eq!(waveform.dimensions.width, 256);
    assert_eq!(waveform.dimensions.height, 256);
    assert_eq!(waveform.format, PixelFormat::Rgba8);
    assert!(!waveform.data.is_empty());

    // 2. Vectorscope
    let vectorscope = render_vectorscope(&test_frame, 256);
    assert_eq!(vectorscope.dimensions.width, 256);
    assert_eq!(vectorscope.dimensions.height, 256);
    assert_eq!(vectorscope.format, PixelFormat::Rgba8);
    assert!(!vectorscope.data.is_empty());

    // 3. RGB Parade
    let parade = render_rgb_parade(&test_frame, 384, 256);
    assert_eq!(parade.dimensions.width, 384);
    assert_eq!(parade.dimensions.height, 256);
    assert_eq!(parade.format, PixelFormat::Rgba8);
    assert!(!parade.data.is_empty());
}
