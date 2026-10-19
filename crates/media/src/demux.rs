use crate::frame::{AudioFrame, VideoFrame};
use iris_core::{Dimensions, IrisResult, PixelFormat, RationalTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    pub codec: String,
    pub dimensions: Dimensions,
    pub frame_rate_num: u64,
    pub frame_rate_den: u64,
    pub duration: RationalTime,
    pub pixel_format: PixelFormat,
    pub bit_rate: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: RationalTime,
    pub bit_rate: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub date: Option<String>,
    pub orientation_degrees: u32,
}

pub trait Demuxer: Send + Sync {
    fn stream_info(&self) -> &StreamInfo;
    fn metadata(&self) -> &MediaMetadata;
    fn seek(&mut self, time: RationalTime) -> IrisResult<()>;
    fn next_video_frame(&mut self) -> IrisResult<Option<VideoFrame>>;
    fn next_audio_frame(&mut self) -> IrisResult<Option<AudioFrame>>;
}

/// A synthetic media demuxer generating deterministic frames for tests and offline previewing.
pub struct SyntheticDemuxer {
    info: StreamInfo,
    metadata: MediaMetadata,
    current_time: RationalTime,
    fps_num: u64,
    fps_den: u64,
}

impl SyntheticDemuxer {
    pub fn new(dimensions: Dimensions, duration: RationalTime, fps_num: u64, fps_den: u64) -> Self {
        let v_info = VideoStreamInfo {
            codec: "synthetic_raw".to_string(),
            dimensions,
            frame_rate_num: fps_num,
            frame_rate_den: fps_den,
            duration,
            pixel_format: PixelFormat::Rgba8,
            bit_rate: 10_000_000,
        };

        let a_info = AudioStreamInfo {
            codec: "pcm_f32".to_string(),
            sample_rate: 48000,
            channels: 2,
            duration,
            bit_rate: 1_536_000,
        };

        Self {
            info: StreamInfo {
                video_streams: vec![v_info],
                audio_streams: vec![a_info],
            },
            metadata: MediaMetadata {
                title: Some("Synthetic Test Media".into()),
                artist: Some("Iris Studio".into()),
                date: Some("2026".into()),
                orientation_degrees: 0,
            },
            current_time: RationalTime::ZERO,
            fps_num,
            fps_den,
        }
    }
}

impl Demuxer for SyntheticDemuxer {
    fn stream_info(&self) -> &StreamInfo {
        &self.info
    }

    fn metadata(&self) -> &MediaMetadata {
        &self.metadata
    }

    fn seek(&mut self, time: RationalTime) -> IrisResult<()> {
        let total_dur = self.info.video_streams[0].duration;
        self.current_time = if time > total_dur {
            total_dur
        } else if time.numerator < 0 {
            RationalTime::ZERO
        } else {
            time
        };
        Ok(())
    }

    fn next_video_frame(&mut self) -> IrisResult<Option<VideoFrame>> {
        let v_info = &self.info.video_streams[0];
        if self.current_time >= v_info.duration {
            return Ok(None);
        }

        let frame_duration = RationalTime::new(self.fps_den as i64, self.fps_num);
        let frame =
            VideoFrame::create_test_pattern(v_info.dimensions, self.current_time, frame_duration);

        self.current_time += frame_duration;
        Ok(Some(frame))
    }

    fn next_audio_frame(&mut self) -> IrisResult<Option<AudioFrame>> {
        let a_info = &self.info.audio_streams[0];
        if self.current_time >= a_info.duration {
            return Ok(None);
        }

        let chunk_duration = 0.04; // 40ms audio chunk
        let frame = AudioFrame::create_test_tone(
            a_info.sample_rate,
            a_info.channels,
            440.0,
            chunk_duration,
            0.5,
        );
        Ok(Some(frame))
    }
}
