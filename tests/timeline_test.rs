use iris_core::{Dimensions, RationalTime, TimeSpan};
use iris_render::{MockSourceResolver, PreviewEngine};
use iris_timeline::{Clip, RippleDeleteCommand, SplitClipCommand, Timeline, Track, TrackKind};

#[test]
fn test_rational_time_arithmetic() {
    let t1 = RationalTime::new(1, 30);
    let t2 = RationalTime::new(1, 60);
    let sum = t1 + t2;
    assert_eq!(sum, RationalTime::new(1, 20));

    let diff = t1 - t2;
    assert_eq!(diff, RationalTime::new(1, 60));

    let timecode = RationalTime::from_seconds(65.5, 1000).to_timecode();
    assert_eq!(timecode, "00:01:05.500");
}

#[test]
fn test_timeline_split_and_undo() {
    let mut timeline = Timeline::new("test_tl", 30, 1);
    let mut track = Track::new("trk_1", "Video Track", TrackKind::Video);

    let clip = Clip::new(
        "clip_orig",
        "asset_1",
        "Scene A",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)),
        RationalTime::ZERO,
    );
    track.clips.push(clip);
    timeline.tracks.push(track);

    let split_cmd = SplitClipCommand {
        track_id: "trk_1".into(),
        clip_id: "clip_orig".into(),
        split_time: RationalTime::new(4, 1),
        new_clip_id: "clip_part2".into(),
    };

    use iris_timeline::Command;
    split_cmd.execute(&mut timeline).expect("Split failed");

    assert_eq!(timeline.tracks[0].clips.len(), 2);
    assert_eq!(
        timeline.tracks[0].clips[0].timeline_range.duration,
        RationalTime::new(4, 1)
    );
    assert_eq!(
        timeline.tracks[0].clips[1].timeline_range.duration,
        RationalTime::new(6, 1)
    );
    assert_eq!(
        timeline.tracks[0].clips[1].timeline_range.start,
        RationalTime::new(4, 1)
    );

    // Test undo
    split_cmd.undo(&mut timeline).expect("Undo split failed");
    assert_eq!(timeline.tracks[0].clips.len(), 1);
    assert_eq!(
        timeline.tracks[0].clips[0].timeline_range.duration,
        RationalTime::new(10, 1)
    );
}

#[test]
fn test_timeline_ripple_delete() {
    let mut timeline = Timeline::new("test_tl", 30, 1);
    let mut track = Track::new("trk_1", "Video Track", TrackKind::Video);

    track.clips.push(Clip::new(
        "c1",
        "a1",
        "Clip 1",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(3, 1)),
        RationalTime::ZERO,
    ));
    track.clips.push(Clip::new(
        "c2",
        "a2",
        "Clip 2 (ToDelete)",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(4, 1)),
        RationalTime::new(3, 1),
    ));
    track.clips.push(Clip::new(
        "c3",
        "a3",
        "Clip 3",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(5, 1)),
        RationalTime::new(7, 1),
    ));
    timeline.tracks.push(track);

    let deleted = timeline.tracks[0].clips[1].clone();
    let cmd = RippleDeleteCommand {
        track_id: "trk_1".into(),
        clip_id: "c2".into(),
        deleted_clip: Some(deleted),
        deleted_idx: 1,
    };

    use iris_timeline::Command;
    cmd.execute(&mut timeline).expect("Ripple delete failed");

    assert_eq!(timeline.tracks[0].clips.len(), 2);
    // Clip 3 should have shifted left from 7s to 3s
    assert_eq!(
        timeline.tracks[0].clips[1].timeline_range.start,
        RationalTime::new(3, 1)
    );
}

#[test]
fn test_preview_render_evaluation() {
    let mut timeline = Timeline::new("test_tl", 30, 1);
    let mut track = Track::new("trk_1", "Video Track", TrackKind::Video);

    track.clips.push(Clip::new(
        "c1",
        "test_video_1",
        "Clip 1",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(5, 1)),
        RationalTime::ZERO,
    ));
    timeline.tracks.push(track);

    let engine = PreviewEngine::new(Dimensions::new(640, 360));
    let resolver = MockSourceResolver::default();

    let frame = engine
        .render_preview(&timeline, RationalTime::new(1, 1), &resolver)
        .expect("Render failed")
        .expect("Frame dropped");

    assert_eq!(frame.dimensions.width, 640);
    assert_eq!(frame.dimensions.height, 360);
    assert_eq!(frame.data.len(), 640 * 360 * 4);
}
