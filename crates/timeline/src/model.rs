use crate::transform::ClipTransform;
use iris_core::{RationalTime, TimeSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Video,
    Audio,
    Overlay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: RationalTime,
    pub value: f32,
}

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
    FadeToBlack,
    Slide,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub kind: TransitionKind,
    pub duration: RationalTime,
    pub track_id: String,
    pub clip_a_id: String,
    pub clip_b_id: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub id: String,
    pub fps_num: u64,
    pub fps_den: u64,
    pub tracks: Vec<Track>,
    pub transitions: Vec<Transition>,
}

impl Timeline {
    pub fn new(id: impl Into<String>, fps_num: u64, fps_den: u64) -> Self {
        Self {
            id: id.into(),
            fps_num,
            fps_den,
            tracks: Vec::new(),
            transitions: Vec::new(),
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
