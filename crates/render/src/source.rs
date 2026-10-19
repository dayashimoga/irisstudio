use iris_core::{Dimensions, IrisResult, RationalTime};
use iris_media::VideoFrame;
use std::collections::HashMap;

pub trait SourceResolver: Send + Sync {
    fn resolve_video_frame(
        &self,
        asset_id: &str,
        source_time: RationalTime,
        target_dimensions: Dimensions,
    ) -> IrisResult<VideoFrame>;
}

pub struct MockSourceResolver {
    patterns: HashMap<String, Dimensions>,
}

impl Default for MockSourceResolver {
    fn default() -> Self {
        let mut patterns = HashMap::new();
        patterns.insert("test_video_1".into(), Dimensions::new(1920, 1080));
        patterns.insert("test_video_2".into(), Dimensions::new(1920, 1080));
        patterns.insert("photo_1".into(), Dimensions::new(1920, 1080));
        Self { patterns }
    }
}

impl SourceResolver for MockSourceResolver {
    fn resolve_video_frame(
        &self,
        asset_id: &str,
        source_time: RationalTime,
        target_dimensions: Dimensions,
    ) -> IrisResult<VideoFrame> {
        let dim = self
            .patterns
            .get(asset_id)
            .copied()
            .unwrap_or(target_dimensions);

        let duration = RationalTime::new(1, 30);
        let frame = VideoFrame::create_test_pattern(dim, source_time, duration);
        Ok(frame)
    }
}
