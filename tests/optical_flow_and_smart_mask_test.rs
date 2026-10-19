use iris_effects::{
    apply_smart_mask, check_forward_backward_consistency, compute_edge_gradient_field,
    estimate_motion_field, generate_slow_motion_sequence, generate_smart_mask, interpolate_frame,
    ContourBoundingBox, OpticalFlowConfig, SeedPoint, SmartMaskParams,
};

#[test]
fn test_optical_flow_motion_estimation_and_subpixel() {
    let width = 64;
    let height = 64;
    let mut frame_0 = vec![0u8; width * height * 4];
    let mut frame_1 = vec![0u8; width * height * 4];

    // Background is dark blue: [10, 15, 30, 255]
    for i in (0..frame_0.len()).step_by(4) {
        frame_0[i] = 10;
        frame_0[i + 1] = 15;
        frame_0[i + 2] = 30;
        frame_0[i + 3] = 255;

        frame_1[i] = 10;
        frame_1[i + 1] = 15;
        frame_1[i + 2] = 30;
        frame_1[i + 3] = 255;
    }

    // Radial isotropic feature in frame_0 centered at (28, 28)
    for y in 16..40 {
        for x in 16..40 {
            let idx = (y * width + x) * 4;
            let dist = ((x as f32 - 28.0).powi(2) + (y as f32 - 28.0).powi(2)).sqrt();
            let val = (255.0 - dist * 18.0).clamp(10.0, 255.0) as u8;
            frame_0[idx] = val;
            frame_0[idx + 1] = val;
            frame_0[idx + 2] = val;
        }
    }

    // Shifted radial feature in frame_1 by (+4, +2) centered at (32, 30)
    for y in 18..42 {
        for x in 20..44 {
            let idx = (y * width + x) * 4;
            let dist = ((x as f32 - 32.0).powi(2) + (y as f32 - 30.0).powi(2)).sqrt();
            let val = (255.0 - dist * 18.0).clamp(10.0, 255.0) as u8;
            frame_1[idx] = val;
            frame_1[idx + 1] = val;
            frame_1[idx + 2] = val;
        }
    }

    let config = OpticalFlowConfig {
        block_size: 8,
        search_radius: 8,
        occlusion_threshold: 1.5,
    };

    let field = estimate_motion_field(&frame_0, &frame_1, width, height, &config);

    // Check block centered at (28, 28) -> block coords (3, 3)
    let vec = field.get(3, 3);
    assert!(
        (vec.dx - 4.0).abs() <= 0.6,
        "Expected dx near 4.0, got {}",
        vec.dx
    );
    assert!(
        (vec.dy - 2.0).abs() <= 0.6,
        "Expected dy near 2.0, got {}",
        vec.dy
    );
}

#[test]
fn test_optical_flow_forward_backward_consistency() {
    let width = 32;
    let height = 32;
    let mut frame_0 = vec![30u8; width * height * 4];
    let mut frame_1 = vec![30u8; width * height * 4];

    // Square shifted diagonally by (+2, +2)
    for y in 10..20 {
        for x in 10..20 {
            let idx0 = (y * width + x) * 4;
            frame_0[idx0] = 220;
            let idx1 = ((y + 2) * width + (x + 2)) * 4;
            frame_1[idx1] = 220;
        }
    }

    let config = OpticalFlowConfig {
        block_size: 8,
        search_radius: 6,
        occlusion_threshold: 1.5,
    };

    let mut forward_field = estimate_motion_field(&frame_0, &frame_1, width, height, &config);
    let backward_field = estimate_motion_field(&frame_1, &frame_0, width, height, &config);

    check_forward_backward_consistency(
        &mut forward_field,
        &backward_field,
        config.occlusion_threshold,
    );

    // The central moving block should be consistent (unoccluded)
    let center_vec = forward_field.get(1, 1);
    assert!(
        !center_vec.is_occluded,
        "Central moving block should be consistent"
    );
}

#[test]
fn test_optical_flow_frame_interpolation() {
    let width = 48;
    let height = 48;
    let mut frame_0 = vec![0u8; width * height * 4];
    let mut frame_1 = vec![0u8; width * height * 4];

    // Background red [200, 20, 20, 255]
    for i in (0..frame_0.len()).step_by(4) {
        frame_0[i] = 200;
        frame_0[i + 1] = 20;
        frame_0[i + 2] = 20;
        frame_0[i + 3] = 255;

        frame_1[i] = 200;
        frame_1[i + 1] = 20;
        frame_1[i + 2] = 20;
        frame_1[i + 3] = 255;
    }

    // Moving green square: in frame 0 at x=10..22, in frame 1 at x=18..30 (shifted +8 px)
    for y in 16..28 {
        for x in 10..22 {
            let idx = (y * width + x) * 4;
            frame_0[idx] = 10;
            frame_0[idx + 1] = 240;
            frame_0[idx + 2] = 10;
        }
        for x in 18..30 {
            let idx = (y * width + x) * 4;
            frame_1[idx] = 10;
            frame_1[idx + 1] = 240;
            frame_1[idx + 2] = 10;
        }
    }

    let config = OpticalFlowConfig {
        block_size: 8,
        search_radius: 12,
        occlusion_threshold: 2.0,
    };

    let forward = estimate_motion_field(&frame_0, &frame_1, width, height, &config);
    let backward = estimate_motion_field(&frame_1, &frame_0, width, height, &config);

    let mut interpolated = vec![0u8; width * height * 4];
    interpolate_frame(
        &frame_0,
        &frame_1,
        &mut interpolated,
        width,
        height,
        &forward,
        &backward,
        0.5,
    );

    // At t=0.5, the green square should be centered at midpoint x=14..26 (10 + 4 = 14)
    let mid_pixel_idx = (22 * width + 20) * 4;
    assert!(
        interpolated[mid_pixel_idx + 1] > 180,
        "Midpoint pixel should be green in interpolated frame, got green={}",
        interpolated[mid_pixel_idx + 1]
    );
}

#[test]
fn test_slow_motion_sequence_generation() {
    let width = 32;
    let height = 32;
    let mut frame_0 = vec![50u8; width * height * 4];
    let mut frame_1 = vec![50u8; width * height * 4];

    // Bright feature moving
    for y in 10..18 {
        for x in 8..16 {
            frame_0[(y * width + x) * 4] = 250;
            frame_1[(y * width + (x + 4)) * 4] = 250;
        }
    }

    let config = OpticalFlowConfig::default();
    let sequence = generate_slow_motion_sequence(&frame_0, &frame_1, width, height, 4, &config);

    assert_eq!(
        sequence.len(),
        4,
        "4x slow-mo multiplier should generate 4 frames"
    );
    for frame in &sequence {
        assert_eq!(frame.len(), width * height * 4);
    }
}

#[test]
fn test_smart_mask_edge_gradient_and_matting() {
    let width = 32;
    let height = 32;
    let mut src = vec![0u8; width * height * 4];

    // Background: Blue [0, 50, 200, 255]
    for i in (0..src.len()).step_by(4) {
        src[i] = 0;
        src[i + 1] = 50;
        src[i + 2] = 200;
        src[i + 3] = 255;
    }

    // Foreground subject: Green square [20, 230, 40, 255] in center (10..22, 10..22)
    for y in 10..22 {
        for x in 10..22 {
            let idx = (y * width + x) * 4;
            src[idx] = 20;
            src[idx + 1] = 230;
            src[idx + 2] = 40;
        }
    }

    // Verify edge gradient detection
    let gradients = compute_edge_gradient_field(&src, width, height);
    let edge_idx = 10 * width + 10;
    assert!(
        gradients[edge_idx] > 0.1,
        "Edge pixel should have significant gradient magnitude"
    );

    // Provide seeds: 1 inside green subject, 1 in blue background
    let seeds = vec![
        SeedPoint {
            x: 16,
            y: 16,
            is_foreground: true,
        },
        SeedPoint {
            x: 2,
            y: 2,
            is_foreground: false,
        },
    ];

    let bbox = ContourBoundingBox {
        min_x: 6,
        min_y: 6,
        max_x: 26,
        max_y: 26,
    };

    let params = SmartMaskParams {
        contrast_threshold: 0.2,
        color_tolerance: 0.3,
        feather_radius: 1,
        invert: false,
        iterations: 4,
    };

    let mask = generate_smart_mask(&src, width, height, &seeds, Some(bbox), &params);

    // Mask should be high inside foreground
    let fg_alpha = mask[16 * width + 16];
    assert!(
        fg_alpha >= 180,
        "Foreground seed region should have high alpha, got {}",
        fg_alpha
    );

    // Mask should be zero outside bounding box
    let outside_alpha = mask[2 * width + 2];
    assert_eq!(
        outside_alpha, 0,
        "Outside bounding box should have zero alpha"
    );

    // Test applying smart mask to RGBA
    let mut out_rgba = vec![0u8; width * height * 4];
    apply_smart_mask(&src, &mask, &mut out_rgba, width, height);

    let fg_pixel_alpha = out_rgba[(16 * width + 16) * 4 + 3];
    assert!(
        fg_pixel_alpha >= 180,
        "Applied output alpha should match mask"
    );

    let bg_pixel_alpha = out_rgba[(2 * width + 2) * 4 + 3];
    assert_eq!(bg_pixel_alpha, 0, "Background output alpha should be zero");
}
