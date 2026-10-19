use crate::transform::ClipTransform;
use iris_core::{RationalTime, TimeSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Video,
    Audio,
    Overlay,
}

pub use crate::keyframes::{Easing, Keyframe};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub asset_id: String,
    pub name: String,
    pub source_range: TimeSpan,
    pub timeline_range: TimeSpan,
    pub transform: ClipTransform,
    pub speed: f32,
    pub volume_keyframes: Vec<Keyframe>,
}

impl Clip {
    pub fn new(
        id: impl Into<String>,
        asset_id: impl Into<String>,
        name: impl Into<String>,
        source_range: TimeSpan,
        timeline_start: RationalTime,
    ) -> Self {
        Self {
            id: id.into(),
            asset_id: asset_id.into(),
            name: name.into(),
            source_range,
            timeline_range: TimeSpan::new(timeline_start, source_range.duration),
            transform: ClipTransform::default(),
            speed: 1.0,
            volume_keyframes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionKind {
    CrossDissolve,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    FadeToBlack,
    DipToColor { r: u8, g: u8, b: u8 },
    Slide,
    PushLeft,
    PushRight,
    IrisRound,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub kind: TransitionKind,
    pub start_time: RationalTime,
    pub duration: RationalTime,
    pub track_id: String,
    pub clip_a_id: String,
    pub clip_b_id: String,
}

impl Transition {
    pub fn new(
        id: impl Into<String>,
        kind: TransitionKind,
        start_time: RationalTime,
        duration: RationalTime,
        track_id: impl Into<String>,
        clip_a_id: impl Into<String>,
        clip_b_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            start_time,
            duration,
            track_id: track_id.into(),
            clip_a_id: clip_a_id.into(),
            clip_b_id: clip_b_id.into(),
        }
    }

    /// Evaluates the normalized progress t in [0.0, 1.0] at a given timeline time.
    /// Returns None if the time is outside the transition span.
    pub fn progress_at(&self, time: RationalTime) -> Option<f32> {
        let end_time = self.start_time + self.duration;
        if time >= self.start_time && time < end_time {
            let dur_sec = self.duration.to_seconds();
            if dur_sec <= 0.0 {
                return Some(1.0);
            }
            let elapsed = (time - self.start_time).to_seconds();
            Some((elapsed / dur_sec).clamp(0.0, 1.0) as f32)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub kind: TrackKind,
    pub muted: bool,
    pub solo: bool,
    pub locked: bool,
    pub volume: f32,
    pub clips: Vec<Clip>,
}

impl Track {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: TrackKind) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            muted: false,
            solo: false,
            locked: false,
            volume: 1.0,
            clips: Vec::new(),
        }
    }

    pub fn sort_clips(&mut self) {
        self.clips.sort_by_key(|c| c.timeline_range.start);
    }
}

use crate::subtitles::SubtitleTrack;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub id: String,
    pub fps_num: u64,
    pub fps_den: u64,
    pub tracks: Vec<Track>,
    pub transitions: Vec<Transition>,
    #[serde(default)]
    pub subtitle_tracks: Vec<SubtitleTrack>,
}

impl Timeline {
    pub fn new(id: impl Into<String>, fps_num: u64, fps_den: u64) -> Self {
        Self {
            id: id.into(),
            fps_num,
            fps_den,
            tracks: Vec::new(),
            transitions: Vec::new(),
            subtitle_tracks: Vec::new(),
        }
    }

    pub fn total_duration(&self) -> RationalTime {
        let mut max_time = RationalTime::ZERO;
        for track in &self.tracks {
            for clip in &track.clips {
                let end = clip.timeline_range.end();
                if end > max_time {
                    max_time = end;
                }
            }
        }
        max_time
    }
}
