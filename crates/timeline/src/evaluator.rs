use crate::model::{Clip, Timeline, TrackKind};
use crate::transform::ClipTransform;
use iris_core::RationalTime;

#[derive(Debug, Clone)]
pub struct ActiveClipRenderState {
    pub clip_id: String,
    pub asset_id: String,
    pub track_kind: TrackKind,
    pub source_time: RationalTime,
    pub transform: ClipTransform,
    pub opacity: f32,
    pub volume: f32,
}

pub struct TimelineEvaluator;

impl TimelineEvaluator {
    pub fn evaluate_at(timeline: &Timeline, time: RationalTime) -> Vec<ActiveClipRenderState> {
        let mut active = Vec::new();

        for track in &timeline.tracks {
            if track.muted {
                continue;
            }

            for clip in &track.clips {
                if clip.timeline_range.contains(time) {
                    let offset = time - clip.timeline_range.start;
                    // Apply speed scaling: source_offset = offset * speed
                    let scaled_num = (offset.numerator as f64 * clip.speed as f64).round() as i64;
                    let scaled_offset = RationalTime::new(scaled_num, offset.denominator);
                    let source_time = clip.source_range.start + scaled_offset;

                    let mut effective_transform = clip.transform;
                    effective_transform.opacity *= clip.transform.opacity;

                    let volume = Self::interpolate_volume(clip, time) * track.volume;

                    active.push(ActiveClipRenderState {
                        clip_id: clip.id.clone(),
                        asset_id: clip.asset_id.clone(),
                        track_kind: track.kind,
                        source_time,
                        transform: effective_transform,
                        opacity: effective_transform.opacity,
                        volume,
                    });
                }
            }
        }

        active
    }

    fn interpolate_volume(clip: &Clip, time: RationalTime) -> f32 {
        if clip.volume_keyframes.is_empty() {
            return 1.0;
        }
        if clip.volume_keyframes.len() == 1 {
            return clip.volume_keyframes[0].value;
        }

        let kfs = &clip.volume_keyframes;
        if time <= kfs[0].time {
            return kfs[0].value;
        }
        if time >= kfs.last().unwrap().time {
            return kfs.last().unwrap().value;
        }

        // Linear interpolation between adjacent keyframes
        for i in 0..kfs.len() - 1 {
            if time >= kfs[i].time && time <= kfs[i + 1].time {
                let t0 = kfs[i].time.to_seconds();
                let t1 = kfs[i + 1].time.to_seconds();
                let cur = time.to_seconds();
                let factor = ((cur - t0) / (t1 - t0).max(1e-6)) as f32;
                return kfs[i].value + (kfs[i + 1].value - kfs[i].value) * factor;
            }
        }

        1.0
    }
}
