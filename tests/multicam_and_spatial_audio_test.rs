use iris_audio::{BFormatFrame, BinauralPanner};
use iris_core::{Dimensions, PixelFormat, RationalTime, TimeSpan};
use iris_effects::{
    aces_filmic, apply_hdr_tonemap, hlg_inverse_oetf, pq_eotf, pq_inverse_eotf, HdrToneMapParams,
    HdrTransferFunction, ToneMappingAlgorithm,
};
use iris_media::VideoFrame;
use iris_render::{render_multicam_grid, GpuComputePipeline, MultiCamAngleView};
use iris_timeline::{CameraAngle, MultiCamClip};
use std::f32::consts::PI;

#[test]
fn test_multicam_clip_switching_and_flattening() {
    let timeline_range = TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)); // 10s
    let mut multicam = MultiCamClip::new(
        "mc_01",
        "Concert Multicam Group",
        timeline_range,
        "angle_wide",
    );

    let angle_wide = CameraAngle::new(
        "angle_wide",
        "Cam A (Wide)",
        "asset_wide",
        RationalTime::ZERO,
    );
    let angle_close = CameraAngle::new(
        "angle_close",
        "Cam B (Close)",
        "asset_close",
        RationalTime::new(1, 1),
    );
    let angle_drone = CameraAngle::new(
        "angle_drone",
        "Cam C (Drone)",
        "asset_drone",
        RationalTime::new(2, 1),
    );

    multicam.add_angle(angle_wide);
    multicam.add_angle(angle_close);
    multicam.add_angle(angle_drone);

    assert_eq!(multicam.angles.len(), 3);

    // Initial state: 0.0s should be default angle (wide)
    let active_0 = multicam.active_angle_at(RationalTime::new(0, 1)).unwrap();
    assert_eq!(active_0.id, "angle_wide");

    // Cut to close-up at 3.0s
    assert!(multicam.switch_angle(RationalTime::new(3, 1), "angle_close"));
    // Cut to drone at 7.0s
    assert!(multicam.switch_angle(RationalTime::new(7, 1), "angle_drone"));

    // Verify angle queries across timeline
    assert_eq!(
        multicam
            .active_angle_at(RationalTime::new(1, 1))
            .unwrap()
            .id,
        "angle_wide"
    );
    assert_eq!(
        multicam
            .active_angle_at(RationalTime::new(3, 1))
            .unwrap()
            .id,
        "angle_close"
    );
    assert_eq!(
        multicam
            .active_angle_at(RationalTime::new(5, 1))
            .unwrap()
            .id,
        "angle_close"
    );
    assert_eq!(
        multicam
            .active_angle_at(RationalTime::new(7, 1))
            .unwrap()
            .id,
        "angle_drone"
    );
    assert_eq!(
        multicam
            .active_angle_at(RationalTime::new(9, 1))
            .unwrap()
            .id,
        "angle_drone"
    );

    // Flatten to standard timeline clips
    let clips = multicam.flatten_to_clips();
    assert_eq!(clips.len(), 3, "Expected 3 flattened clips from 2 cuts");

    // Clip 1: 0s to 3s (angle_wide, sync_offset 0s)
    assert_eq!(clips[0].asset_id, "asset_wide");
    assert_eq!(clips[0].timeline_range.start, RationalTime::ZERO);
    assert_eq!(clips[0].timeline_range.duration, RationalTime::new(3, 1));
    assert_eq!(clips[0].source_range.start, RationalTime::ZERO);

    // Clip 2: 3s to 7s (angle_close, sync_offset 1s => source starts at 1s + (3s - 0s) = 4s)
    assert_eq!(clips[1].asset_id, "asset_close");
    assert_eq!(clips[1].timeline_range.start, RationalTime::new(3, 1));
    assert_eq!(clips[1].timeline_range.duration, RationalTime::new(4, 1));
    assert_eq!(clips[1].source_range.start, RationalTime::new(4, 1));

    // Clip 3: 7s to 10s (angle_drone, sync_offset 2s => source starts at 2s + (7s - 0s) = 9s)
    assert_eq!(clips[2].asset_id, "asset_drone");
    assert_eq!(clips[2].timeline_range.start, RationalTime::new(7, 1));
    assert_eq!(clips[2].timeline_range.duration, RationalTime::new(3, 1));
    assert_eq!(clips[2].source_range.start, RationalTime::new(9, 1));
}

#[test]
fn test_multicam_quad_grid_rendering() {
    let dims = Dimensions::new(640, 360);
    let mut frame1 = VideoFrame::new(
        dims,
        PixelFormat::Rgba8,
        RationalTime::ZERO,
        RationalTime::new(1, 30),
    );
    let mut frame2 = VideoFrame::new(
        dims,
        PixelFormat::Rgba8,
        RationalTime::ZERO,
        RationalTime::new(1, 30),
    );
    let mut frame3 = VideoFrame::new(
        dims,
        PixelFormat::Rgba8,
        RationalTime::ZERO,
        RationalTime::new(1, 30),
    );
    let mut frame4 = VideoFrame::new(
        dims,
        PixelFormat::Rgba8,
        RationalTime::ZERO,
        RationalTime::new(1, 30),
    );

    // Tint each angle distinctly: Cam 1 Red, Cam 2 Green, Cam 3 Blue, Cam 4 Magenta
    for chunk in frame1.data.as_chunks_mut::<4>().0 {
        chunk[0] = 220;
        chunk[3] = 255;
    }
    for chunk in frame2.data.as_chunks_mut::<4>().0 {
        chunk[1] = 220;
        chunk[3] = 255;
    }
    for chunk in frame3.data.as_chunks_mut::<4>().0 {
        chunk[2] = 220;
        chunk[3] = 255;
    }
    for chunk in frame4.data.as_chunks_mut::<4>().0 {
        chunk[0] = 220;
        chunk[2] = 220;
        chunk[3] = 255;
    }

    let angles = [
        MultiCamAngleView {
            frame: &frame1,
            name: "WIDE",
            is_active: false,
        },
        MultiCamAngleView {
            frame: &frame2,
            name: "CLOSE-UP",
            is_active: true,
        }, // On-air angle
        MultiCamAngleView {
            frame: &frame3,
            name: "DRONE",
            is_active: false,
        },
        MultiCamAngleView {
            frame: &frame4,
            name: "STAGE",
            is_active: false,
        },
    ];

    let grid = render_multicam_grid(&angles, 1280, 720);
    assert_eq!(grid.dimensions.width, 1280);
    assert_eq!(grid.dimensions.height, 720);
    assert_eq!(grid.data.len(), 1280 * 720 * 4);

    // Quadrant 2 (top-right, x: 640..1280, y: 0..360) is the active angle (Cam 2)
    // The top-right edge should contain the active orange border ([255, 138, 0])
    let offset = (10 * grid.stride) + (700 * 4); // inside active top-right quad
                                                 // Green tint from Cam 2 should be present
    assert!(
        grid.data[offset + 1] > 150,
        "Green channel should be dominant in Cam 2 quadrant"
    );
}

#[test]
fn test_ambisonic_and_binaural_spatial_audio() {
    // 1. Ambisonics B-format point source encoding
    let left_azim = PI / 2.0; // 90 degrees left
    let b_frame_left = BFormatFrame::from_point_source(1.0, left_azim, 0.0);
    assert!((b_frame_left.w - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
    assert!(
        (b_frame_left.x).abs() < 1e-4,
        "X should be ~0 for 90 deg lateral"
    );
    assert!(
        (b_frame_left.y - 1.0).abs() < 1e-4,
        "Y should be 1.0 for 90 deg left"
    );
    assert!(
        (b_frame_left.z).abs() < 1e-4,
        "Z should be 0 on horizontal plane"
    );

    // 2. Ambisonics B-format decoding to stereo
    let mut stereo_decoded = [0.0f32; 2];
    BinauralPanner::decode_bformat_to_stereo(&[b_frame_left], &mut stereo_decoded);
    // Left ear should receive stronger signal than right ear
    assert!(
        stereo_decoded[0] > stereo_decoded[1],
        "Left ear signal should dominate for 90 deg left"
    );

    // 3. 5.1 Surround decode
    let mut surround_5_1 = [0.0f32; 6];
    BinauralPanner::decode_bformat_to_5_1(&[b_frame_left], &mut surround_5_1);
    // Left side (Ls) and Left (L) channels should receive higher energy than right channels
    assert!(
        surround_5_1[0] > surround_5_1[1],
        "5.1 Left should exceed Right"
    );

    // 4. Binaural 3D Virtual Panner with Woodworth spherical ITD & ILD
    let mut panner = BinauralPanner::new(48000);
    let mono_in = vec![0.8f32; 480]; // 10ms of audio
    let mut binaural_out = vec![0.0f32; 960]; // 480 stereo frames

    panner.process_mono_to_binaural(&mono_in, &mut binaural_out, 90.0, 0.0, 1.5);

    // Sum left and right channel energies
    let mut left_energy = 0.0f32;
    let mut right_energy = 0.0f32;
    for i in 0..480 {
        left_energy += binaural_out[i * 2].abs();
        right_energy += binaural_out[i * 2 + 1].abs();
    }

    assert!(
        left_energy > right_energy * 1.5,
        "Left ear energy ({}) must significantly exceed right ear ({}) for sound at 90 deg left",
        left_energy,
        right_energy
    );
}

#[test]
fn test_hdr_transfer_functions_and_aces_tonemapping() {
    // 1. PQ EOTF roundtrip tests at reference levels
    let nits_100 = pq_eotf(pq_inverse_eotf(100.0));
    assert!(
        (nits_100 - 100.0).abs() < 1.0,
        "PQ 100 nits roundtrip mismatch: {}",
        nits_100
    );

    let nits_1000 = pq_eotf(pq_inverse_eotf(1000.0));
    assert!(
        (nits_1000 - 1000.0).abs() < 5.0,
        "PQ 1000 nits roundtrip mismatch: {}",
        nits_1000
    );

    let nits_4000 = pq_eotf(pq_inverse_eotf(4000.0));
    assert!(
        (nits_4000 - 4000.0).abs() < 20.0,
        "PQ 4000 nits roundtrip mismatch: {}",
        nits_4000
    );

    // 2. HLG Inverse OETF
    let hlg_mid = hlg_inverse_oetf(0.5);
    assert!((hlg_mid - (0.25 / 3.0)).abs() < 1e-4);

    // 3. ACES Filmic curve properties
    assert_eq!(aces_filmic(0.0), 0.0);
    assert!(aces_filmic(1.0) > 0.7);
    assert!(
        aces_filmic(100.0) <= 1.0,
        "ACES curve must asymptote to <= 1.0"
    );

    // 4. Test apply_hdr_tonemap on RGBA image buffer
    let mut hdr_pixels = vec![
        240, 240, 240, 255, // Very high luminance PQ pixel (~4000 nits)
        128, 128, 128, 255, // Mid-tone PQ pixel
    ];

    let params = HdrToneMapParams {
        transfer_function: HdrTransferFunction::Pq,
        algorithm: ToneMappingAlgorithm::AcesFilmic,
        exposure_stops: 0.0,
        target_nits: 100.0,
        white_point: 3.0,
    };

    apply_hdr_tonemap(&mut hdr_pixels, &params);

    // Highlight pixel should be mapped cleanly without clipping to full saturated white or overflow
    assert!(
        hdr_pixels[0] > 200,
        "High dynamic range highlight should remain bright SDR"
    );
    assert!(
        hdr_pixels[0] >= hdr_pixels[4],
        "Highlight should be brighter than midtone"
    );
}

#[test]
fn test_gpu_compute_pipeline_tonemap_and_composite() {
    let pipeline = GpuComputePipeline::new();
    assert!(pipeline.info.is_hardware_accelerated);

    let in_pixels = vec![100, 150, 200, 255, 50, 75, 100, 255];
    let mut out_pixels = vec![0u8; 8];

    pipeline.dispatch_tonemap(&in_pixels, &mut out_pixels, 0.5);
    assert_ne!(out_pixels[0], 0, "Tone mapped pixels should be non-zero");

    // Test composite blending
    let bg_pixels = vec![0, 0, 0, 255];
    let fg_pixels = vec![200, 200, 200, 255];
    let mut comp_out = vec![0u8; 4];

    pipeline.dispatch_composite(&bg_pixels, &fg_pixels, &mut comp_out, 0.5);
    // 50% blend of [200, 200, 200] over black should be ~100
    assert!((comp_out[0] as i32 - 100).abs() <= 2);
}
