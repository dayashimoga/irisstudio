use iris_core::{Dimensions, RationalTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Mp4,
    WebM,
    JpegSequence,
    PngSequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportPreset {
    YouTube4k,
    Web1080p,
    SocialReels9x16,
    LosslessMaster,
}

impl ExportPreset {
    pub fn to_config(self, output_path: impl Into<String>) -> ExportConfig {
        match self {
            Self::YouTube4k => ExportConfig {
                output_path: output_path.into(),
                format: ExportFormat::Mp4,
                dimensions: Dimensions::new(3840, 2160),
                fps_num: 60,
                fps_den: 1,
                video_bitrate_kbps: 45_000,
                audio_bitrate_kbps: 320,
                range_in: None,
                range_out: None,
            },
            Self::Web1080p => ExportConfig {
                output_path: output_path.into(),
                format: ExportFormat::Mp4,
                dimensions: Dimensions::new(1920, 1080),
                fps_num: 30,
                fps_den: 1,
                video_bitrate_kbps: 10_000,
                audio_bitrate_kbps: 192,
                range_in: None,
                range_out: None,
            },
            Self::SocialReels9x16 => ExportConfig {
                output_path: output_path.into(),
                format: ExportFormat::Mp4,
                dimensions: Dimensions::new(1080, 1920),
                fps_num: 60,
                fps_den: 1,
                video_bitrate_kbps: 15_000,
                audio_bitrate_kbps: 192,
                range_in: None,
                range_out: None,
            },
            Self::LosslessMaster => ExportConfig {
                output_path: output_path.into(),
                format: ExportFormat::WebM,
                dimensions: Dimensions::new(1920, 1080),
                fps_num: 60,
                fps_den: 1,
                video_bitrate_kbps: 80_000,
                audio_bitrate_kbps: 320,
                range_in: None,
                range_out: None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportConfig {
    pub output_path: String,
    pub format: ExportFormat,
    pub dimensions: Dimensions,
    pub fps_num: u64,
    pub fps_den: u64,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub range_in: Option<RationalTime>,
    pub range_out: Option<RationalTime>,
}
