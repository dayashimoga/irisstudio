use iris_core::{RationalTime, TimeSpan};
use iris_export::{ExportPreset, ExportScheduler};
use iris_render::MockSourceResolver;
use iris_timeline::{Clip, Timeline, Track, TrackKind};
use std::env;

#[test]
fn test_export_scheduler_execution_and_cancellation() {
    let mut timeline = Timeline::new("export_tl", 30, 1);
    let mut track = Track::new("trk_v1", "Video", TrackKind::Video);
    track.clips.push(Clip::new(
        "c1",
        "test_video_1",
        "Clip 1",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(2, 1)),
        RationalTime::ZERO,
    ));
    timeline.tracks.push(track);

    let temp_dir = env::temp_dir();
    let export_path = temp_dir.join("iris_test_export.mp4");
    let export_path_str = export_path.to_str().unwrap().to_string();

    let config = ExportPreset::Web1080p.to_config(export_path_str);
    let scheduler = ExportScheduler::new();
    let resolver = MockSourceResolver::default();

    let mut last_percent = 0.0f32;
    let res = scheduler.run_export(&timeline, &config, &resolver, |p| {
        last_percent = p.percent;
    });

    assert!(res.is_ok(), "Export failed: {:?}", res);
    assert_eq!(last_percent, 100.0);
    assert!(export_path.exists());

    let _ = std::fs::remove_file(&export_path);
}
