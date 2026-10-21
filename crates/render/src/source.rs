use iris_core::{Dimensions, IrisResult, RationalTime};
use iris_media::{RealMediaDecoder, VideoFrame};
use std::collections::HashMap;
use std::sync::RwLock;

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

/// Real production source resolver linking project assets to media files on disk.
pub struct RealSourceResolver {
    assets: RwLock<HashMap<String, String>>, // asset_id -> file_path
    mock: MockSourceResolver,
}

impl Default for RealSourceResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl RealSourceResolver {
    pub fn new() -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
            mock: MockSourceResolver::default(),
        }
    }

    pub fn register_asset(&self, asset_id: impl Into<String>, file_path: impl Into<String>) {
        if let Ok(mut map) = self.assets.write() {
            map.insert(asset_id.into(), file_path.into());
        }
    }

    pub fn remove_asset(&self, asset_id: &str) {
        if let Ok(mut map) = self.assets.write() {
            map.remove(asset_id);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut map) = self.assets.write() {
            map.clear();
        }
    }
}

impl SourceResolver for RealSourceResolver {
    fn resolve_video_frame(
        &self,
        asset_id: &str,
        source_time: RationalTime,
        target_dimensions: Dimensions,
    ) -> IrisResult<VideoFrame> {
        let path_opt = self
            .assets
            .read()
            .ok()
            .and_then(|m| m.get(asset_id).cloned());
        if let Some(file_path) = path_opt {
            RealMediaDecoder::decode_video_frame(&file_path, source_time, Some(target_dimensions))
        } else {
            self.mock
                .resolve_video_frame(asset_id, source_time, target_dimensions)
        }
    }
}
