use iris_core::{Dimensions, PixelFormat, RationalTime};
use iris_effects::{apply_chroma_key, ChromaKeyParams, MotionTracker, TrackPoint};
use iris_media::{HardwareDeviceContext, HardwareSurfacePool, VideoFrame};
use iris_render::{render_motion_title, TitleTemplate};

#[test]
fn test_chroma_key_green_and_blue_screen() {
    // 4x4 image
    let mut pixels = vec![
        // Row 0: Pure green background
        0, 255, 0, 255, 0, 240, 20, 255, // Row 1: Foreground subject (skin tone)
        220, 160, 120, 255, 200, 150, 100, 255,
    ];

    let params = ChromaKeyParams::green_screen();
    apply_chroma_key(&mut pixels, &params);

    // Green pixels should be keyed out (alpha = 0)
    assert_eq!(pixels[3], 0, "Pure green pixel should be keyed to 0 alpha");
    assert_eq!(
        pixels[7], 0,
        "Slightly perturbed green should be keyed to 0 alpha"
    );

    // Foreground pixels should remain opaque
    assert_eq!(pixels[11], 255, "Foreground subject should remain opaque");
    assert_eq!(pixels[15], 255, "Foreground subject should remain opaque");

    // Test Blue screen keying
    let mut blue_pixels = vec![10, 20, 250, 255, 220, 160, 120, 255];
    let blue_params = ChromaKeyParams::blue_screen();
    apply_chroma_key(&mut blue_pixels, &blue_params);
    assert_eq!(blue_pixels[3], 0, "Blue pixel should be keyed out");
    assert_eq!(
        blue_pixels[7], 255,
        "Foreground pixel in blue screen should remain opaque"
    );
}

#[test]
fn test_motion_tracker_ncc_and_smoothing() {
    let width = 64;
    let height = 64;

    // Create frame A with a distinctive white square at center (28, 28) to (36, 36)
    let mut frame_a = vec![20u8; width * height * 4];
    for y in 26..38 {
        for x in 26..38 {
            let offset = (y * width + x) * 4;
            frame_a[offset] = 240;
            frame_a[offset + 1] = 240;
            frame_a[offset + 2] = 240;
            frame_a[offset + 3] = 255;
        }
    }

    // Create frame B with the square translated by (+6, +4) -> center at (38, 36)
    let mut frame_b = vec![20u8; width * height * 4];
    for y in (26 + 4)..(38 + 4) {
        for x in (26 + 6)..(38 + 6) {
            let offset = (y * width + x) * 4;
            frame_b[offset] = 240;
            frame_b[offset + 1] = 240;
            frame_b[offset + 2] = 240;
            frame_b[offset + 3] = 255;
        }
    }

    let tracker = MotionTracker::new(16, 0.5);
    let tracked = tracker.track_between_frames(&frame_a, &frame_b, width, height, (32.0, 32.0), 16);

    // Tracked center should be near (38.0, 36.0)
    assert!(
        (tracked.center_x - 38.0).abs() <= 1.0,
        "Tracked X center mismatch: {}",
        tracked.center_x
    );
    assert!(
        (tracked.center_y - 36.0).abs() <= 1.0,
        "Tracked Y center mismatch: {}",
        tracked.center_y
    );
    assert!(
        tracked.confidence > 0.8,
        "Confidence score should be high: {}",
        tracked.confidence
    );

    // Test trajectory smoothing
    let raw_trajectory = vec![
        TrackPoint {
            frame_index: 0,
            center_x: 10.0,
            center_y: 10.0,
            confidence: 0.9,
        },
        TrackPoint {
            frame_index: 1,
            center_x: 20.0,
            center_y: 12.0,
            confidence: 0.9,
        }, // jitter jump
        TrackPoint {
            frame_index: 2,
            center_x: 12.0,
            center_y: 11.0,
            confidence: 0.9,
        },
    ];
    let smoothed = tracker.smooth_trajectory(&raw_trajectory);
    assert_eq!(smoothed.len(), 3);
    // Middle point should be smoothed down from 20.0
    assert!(smoothed[1].center_x < 18.0);
}

#[test]
fn test_motion_title_templates_rendering() {
    let mut frame = VideoFrame::new(
        Dimensions::new(640, 360),
        PixelFormat::Rgba8,
        RationalTime::new(0, 30),
        RationalTime::new(1, 30),
    );

    let templates = [
        TitleTemplate::LowerThirdNews,
        TitleTemplate::MinimalClean,
        TitleTemplate::CinematicCenter,
        TitleTemplate::CalloutBox,
    ];

    for &tmpl in &templates {
        // Test entry phase (0.1), steady phase (0.5), exit phase (0.9)
        for &prog in &[0.1f32, 0.5, 0.9] {
            render_motion_title(
                &mut frame,
                "Iris Studio",
                "Advanced Motion Graphics",
                tmpl,
                prog,
            );
            // Verify non-empty frame data
            assert!(!frame.data.is_empty());
        }
    }
}

#[test]
fn test_hardware_device_detection_and_surface_pool() {
    let ctx = HardwareDeviceContext::detect();
    assert!(ctx.max_width >= 1920);
    assert!(ctx.max_height >= 1080);
    assert!(ctx.max_concurrent_streams >= 1);
    assert!(!ctx.device_name.is_empty());

    // Test surface pool ring buffer
    let mut pool = HardwareSurfacePool::new(8, 1920, 1080);
    assert_eq!(pool.free_count(), 8);
    assert_eq!(pool.active_count(), 0);

    let surf1 = pool.acquire_surface().expect("Should acquire surface 1");
    let surf2 = pool.acquire_surface().expect("Should acquire surface 2");
    assert_eq!(pool.free_count(), 6);
    assert_eq!(pool.active_count(), 2);
    assert_ne!(surf1, surf2);

    pool.release_surface(surf1);
    assert_eq!(pool.free_count(), 7);
    assert_eq!(pool.active_count(), 1);

    pool.release_surface(surf2);
    assert_eq!(pool.free_count(), 8);
    assert_eq!(pool.active_count(), 0);
}
