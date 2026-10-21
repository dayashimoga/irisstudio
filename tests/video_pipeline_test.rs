use iris_core::{Dimensions, PixelFormat, RationalTime, TimeSpan};
use iris_effects::{calculate_auto_enhance, Preset, PresetKind};
use iris_export::{ExportPreset, StreamCopyOptimizer, StreamProcessingMode};
use iris_media::{RealMediaDecoder, VideoFrame};
use iris_render::{
    render_subtitles_on_frame, render_transition, Compositor, MockSourceResolver,
    RealSourceResolver,
};
use iris_timeline::{
    interpolate_keyframes, Clip, Easing, Keyframe, SubtitleAlignment, SubtitleItem, SubtitleStyle,
    SubtitleTrack, Timeline, Track, TrackKind, Transition, TransitionKind,
};

fn create_solid_frame(dim: Dimensions, color: [u8; 4]) -> VideoFrame {
    let mut frame = VideoFrame::new(
        dim,
        PixelFormat::Rgba8,
        RationalTime::ZERO,
        RationalTime::new(1, 30),
    );
    for chunk in frame.data.as_chunks_mut::<4>().0 {
        chunk.copy_from_slice(&color);
    }
    frame
}

#[test]
fn test_video_transitions_rasterization() {
    let dim = Dimensions::new(64, 64);
    let red_frame = create_solid_frame(dim, [255, 0, 0, 255]);
    let blue_frame = create_solid_frame(dim, [0, 0, 255, 255]);
    let mut dst = create_solid_frame(dim, [0, 0, 0, 255]);

    let kinds = [
        TransitionKind::CrossDissolve,
        TransitionKind::FadeToBlack,
        TransitionKind::DipToColor {
            r: 255,
            g: 255,
            b: 255,
        },
        TransitionKind::WipeLeft,
        TransitionKind::WipeRight,
        TransitionKind::WipeUp,
        TransitionKind::WipeDown,
        TransitionKind::Slide,
        TransitionKind::PushLeft,
        TransitionKind::PushRight,
        TransitionKind::IrisRound,
        TransitionKind::ZoomIn,
        TransitionKind::ZoomOut,
    ];

    for kind in kinds {
        // Test at midpoint t = 0.5
        let res = render_transition(&red_frame, &blue_frame, kind, 0.5, &mut dst);
        assert!(res.is_ok(), "Failed rendering transition {:?}", kind);
        assert_eq!(dst.data.len(), 64 * 64 * 4);

        // Alpha channel must stay fully opaque 255
        for chunk in dst.data.as_chunks::<4>().0 {
            assert_eq!(chunk[3], 255);
        }

        // Test at boundary t = 0.0 (near or equal to frame A)
        let res_start = render_transition(&red_frame, &blue_frame, kind, 0.0, &mut dst);
        assert!(res_start.is_ok());

        // Test at boundary t = 1.0 (near or equal to frame B)
        let res_end = render_transition(&red_frame, &blue_frame, kind, 1.0, &mut dst);
        assert!(res_end.is_ok());
    }
}

#[test]
fn test_compositor_transition_and_subtitles() {
    let dim = Dimensions::new(320, 240);
    let compositor = Compositor::new(dim);
    let mut timeline = Timeline::new("tl_test", 30, 1);

    let mut track = Track::new("trk_v", "Video", TrackKind::Video);
    let clip1 = Clip::new(
        "c1",
        "asset1",
        "Clip 1",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(4, 1)),
        RationalTime::ZERO,
    );
    let clip2 = Clip::new(
        "c2",
        "asset2",
        "Clip 2",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(4, 1)),
        RationalTime::new(4, 1),
    );
    track.clips.push(clip1);
    track.clips.push(clip2);
    timeline.tracks.push(track);

    // Transition between clip 1 and clip 2 at 3.5s to 4.5s
    let transition = Transition::new(
        "tr1",
        TransitionKind::IrisRound,
        RationalTime::new(7, 2), // 3.5s
        RationalTime::new(1, 1), // 1.0s duration
        "trk_v",
        "c1",
        "c2",
    );
    timeline.transitions.push(transition);

    // Add subtitle track
    let mut sub_track = SubtitleTrack::new("sub_trk", "Captions");
    sub_track.add_item(SubtitleItem::new(
        "s1",
        RationalTime::new(1, 1),
        RationalTime::new(5, 1),
        "Iris Studio Video Engine",
    ));
    timeline.subtitle_tracks.push(sub_track);

    let resolver = MockSourceResolver::default();

    // Render frame during transition at t = 4.0s (midpoint of transition)
    let frame = compositor
        .render_frame(&timeline, RationalTime::new(4, 1), &resolver)
        .expect("Render transition frame should succeed");

    assert_eq!(frame.dimensions, dim);
    assert_eq!(frame.data.len(), 320 * 240 * 4);
}

#[test]
fn test_subtitle_srt_and_vtt_parsing_roundtrip() {
    let srt_sample = "1\n00:00:01,000 --> 00:00:04,500\nHello World\n\n2\n00:00:05,250 --> 00:00:09,000\nSecond Subtitle Line\nMulti-line text\n\n";
    let track = SubtitleTrack::parse_srt(srt_sample).expect("SRT parse should succeed");
    assert_eq!(track.items.len(), 2);
    assert_eq!(track.items[0].text, "Hello World");
    assert_eq!(track.items[0].start_time, RationalTime::new(1000, 1000));
    assert_eq!(track.items[0].end_time, RationalTime::new(4500, 1000));
    assert_eq!(track.items[1].start_time, RationalTime::new(5250, 1000));

    // Roundtrip to SRT
    let exported_srt = track.to_srt();
    assert!(exported_srt.contains("00:00:01,000 --> 00:00:04,500"));
    assert!(exported_srt.contains("Hello World"));

    // WebVTT roundtrip
    let exported_vtt = track.to_vtt();
    assert!(exported_vtt.starts_with("WEBVTT"));
    assert!(exported_vtt.contains("00:00:01.000 --> 00:00:04.500"));

    let vtt_parsed = SubtitleTrack::parse_vtt(&exported_vtt).expect("VTT parse should succeed");
    assert_eq!(vtt_parsed.items.len(), 2);
    assert_eq!(vtt_parsed.items[0].text, "Hello World");
}

#[test]
fn test_subtitle_burn_in_overlay() {
    let dim = Dimensions::new(400, 200);
    let mut frame = create_solid_frame(dim, [0, 0, 0, 255]);

    let mut track = SubtitleTrack::new("st", "Track 1");
    let mut item = SubtitleItem::new(
        "sub1",
        RationalTime::ZERO,
        RationalTime::new(5, 1),
        "Title Card Text",
    );
    item.style = Some(SubtitleStyle {
        font_size: 16,
        text_color: [255, 255, 0, 255], // Yellow
        bg_color: Some([0, 0, 128, 200]),
        alignment: SubtitleAlignment::Center,
        margin_bottom: 20,
    });
    track.add_item(item);

    let res = render_subtitles_on_frame(&mut frame, &[track], RationalTime::new(2, 1));
    assert!(res.is_ok());

    // Verify non-zero color where text/bg was rendered
    let has_yellow_or_bg = frame
        .data
        .as_chunks::<4>()
        .0
        .iter()
        .any(|p| p[0] > 0 || p[2] > 0);
    assert!(has_yellow_or_bg);
}

#[test]
fn test_keyframe_easing_curves() {
    let t0 = RationalTime::ZERO;
    let t1 = RationalTime::new(1, 1); // 1.0s

    // Linear keyframe
    let kfs_linear = vec![
        Keyframe::with_easing(t0, 0.0, Easing::Linear),
        Keyframe::with_easing(t1, 100.0, Easing::Linear),
    ];
    let val_mid_lin = interpolate_keyframes(&kfs_linear, RationalTime::new(1, 2), 0.0);
    assert!((val_mid_lin - 50.0).abs() < 1e-4);

    // EaseIn (quadratic acceleration) -> at 0.5s factor is 0.25 -> val is 25.0
    let kfs_ease_in = vec![
        Keyframe::with_easing(t0, 0.0, Easing::EaseIn),
        Keyframe::with_easing(t1, 100.0, Easing::EaseIn),
    ];
    let val_mid_in = interpolate_keyframes(&kfs_ease_in, RationalTime::new(1, 2), 0.0);
    assert!((val_mid_in - 25.0).abs() < 1e-3);

    // EaseOut (deceleration) -> at 0.5s factor is 0.75 -> val is 75.0
    let kfs_ease_out = vec![
        Keyframe::with_easing(t0, 0.0, Easing::EaseOut),
        Keyframe::with_easing(t1, 100.0, Easing::EaseOut),
    ];
    let val_mid_out = interpolate_keyframes(&kfs_ease_out, RationalTime::new(1, 2), 0.0);
    assert!((val_mid_out - 75.0).abs() < 1e-3);

    // Cubic Bezier (0.25, 0.1, 0.25, 1.0) - CSS ease
    let kfs_bezier = vec![
        Keyframe::with_easing(t0, 0.0, Easing::CubicBezier(0.25, 0.1, 0.25, 1.0)),
        Keyframe::with_easing(t1, 100.0, Easing::Linear),
    ];
    let val_bezier = interpolate_keyframes(&kfs_bezier, RationalTime::new(1, 2), 0.0);
    assert!(val_bezier > 25.0 && val_bezier < 90.0);
}

#[test]
fn test_stream_copy_optimizer() {
    let mut timeline = Timeline::new("tl_copy", 30, 1);
    let mut track = Track::new("trk1", "V1", TrackKind::Video);

    // Clip 1: default transform, normal speed -> eligible for stream copy
    let clip1 = Clip::new(
        "c1",
        "asset_1",
        "Scene 1",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)),
        RationalTime::ZERO,
    );

    // Clip 2: rotated 45 deg -> requires re-encoding
    let mut clip2 = Clip::new(
        "c2",
        "asset_2",
        "Scene 2",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)),
        RationalTime::new(10, 1),
    );
    clip2.transform.rotation_deg = 45.0;

    track.clips.push(clip1);
    track.clips.push(clip2);
    timeline.tracks.push(track);

    let config = ExportPreset::Web1080p.to_config("out.mp4");
    let plan = StreamCopyOptimizer::analyze_timeline(&timeline, &config);

    assert_eq!(plan.segments.len(), 2);
    assert!(matches!(
        plan.segments[0].mode,
        StreamProcessingMode::StreamCopy { .. }
    ));
    assert!(matches!(
        plan.segments[1].mode,
        StreamProcessingMode::ReEncode { .. }
    ));

    // 10s out of 20s = 50% stream copy
    assert!((plan.stream_copy_percentage - 50.0).abs() < 1.0);
    assert!(plan.estimated_speedup > 10.0);
}

#[test]
fn test_real_video_decode_and_source_resolver_composition() {
    let mp4_path = "tests/fixtures/test_video_360p.mp4";
    if std::path::Path::new(mp4_path).exists() {
        // 1. Probe real MP4
        let probe = RealMediaDecoder::probe_file(mp4_path);
        assert!(probe.is_valid);
        assert_eq!(probe.width, 640);
        assert_eq!(probe.height, 360);
        assert!(probe.has_video);
        assert!(probe.has_audio);
        assert!(probe.duration_seconds >= 3.8);

        // 2. Decode real video frame
        let frame_res =
            RealMediaDecoder::decode_video_frame(mp4_path, RationalTime::new(1, 2), None);
        assert!(frame_res.is_ok());
        let frame = frame_res.unwrap();
        assert_eq!(frame.dimensions.width, 640);
        assert_eq!(frame.dimensions.height, 360);
        assert_eq!(frame.data.len(), 640 * 360 * 4);

        // 3. Test RealSourceResolver in Compositor
        let resolver = RealSourceResolver::new();
        resolver.register_asset("clip_real_asset_1", mp4_path);

        let mut timeline = Timeline::new("tl_real_test", 30, 1);
        let mut track = Track::new("trk_v_real", "Video Real", TrackKind::Video);
        let clip = Clip::new(
            "clip_1",
            "clip_real_asset_1",
            "Test 360p Clip",
            TimeSpan::new(RationalTime::ZERO, RationalTime::new(4, 1)),
            RationalTime::ZERO,
        );
        track.clips.push(clip);
        timeline.tracks.push(track);

        let compositor = Compositor::new(Dimensions::new(640, 360));
        let comp_frame = compositor.render_frame(&timeline, RationalTime::new(1, 1), &resolver);
        assert!(comp_frame.is_ok());
        let cf = comp_frame.unwrap();
        assert_eq!(cf.dimensions.width, 640);
        assert_eq!(cf.dimensions.height, 360);
        assert_eq!(cf.data.len(), 640 * 360 * 4);
    }
}

#[test]
fn test_preset_engine_and_auto_enhance() {
    // 1. Video Looks
    let video_looks = Preset::builtin_video_looks();
    assert_eq!(video_looks.len(), 15);
    let cinema_warm = video_looks
        .iter()
        .find(|l| l.name == "Cinema Warm")
        .unwrap();
    assert_eq!(cinema_warm.kind, PresetKind::LookPreset);
    assert!(cinema_warm.adjustments.contrast > 0.0);
    assert!(cinema_warm.lut_intensity > 0.5);

    let sports = video_looks
        .iter()
        .find(|l| l.name == "Sports/Action")
        .unwrap();
    assert!(sports.adjustments.vibrance >= 20.0);

    // Blending
    let half_sports = sports.blend_with_intensity(0.5);
    assert!((half_sports.adjustments.vibrance - sports.adjustments.vibrance * 0.5).abs() < 1e-4);

    // 2. Photo Presets
    let photo_presets = Preset::builtin_photo_presets();
    assert_eq!(photo_presets.len(), 15);
    for p in &photo_presets {
        assert!(p.validate().is_ok());
    }

    // 3. Auto Enhance on solid gray
    let gray_pixels = vec![128u8; 64 * 64 * 4];
    let auto_adj = calculate_auto_enhance(&gray_pixels, 64, 64);
    assert!(auto_adj.exposure.abs() < 1.0);
}
