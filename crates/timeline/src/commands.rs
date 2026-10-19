use crate::model::{Clip, Timeline};
use iris_core::{IrisError, IrisResult, RationalTime, TimeSpan};
use serde::{Deserialize, Serialize};

pub trait Command {
    fn execute(&self, timeline: &mut Timeline) -> IrisResult<()>;
    fn undo(&self, timeline: &mut Timeline) -> IrisResult<()>;
    fn description(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitClipCommand {
    pub track_id: String,
    pub clip_id: String,
    pub split_time: RationalTime,
    pub new_clip_id: String,
}

impl Command for SplitClipCommand {
    fn execute(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| {
                IrisError::TimelineError(format!("Track {} not found", self.track_id))
            })?;

        let clip_idx = track
            .clips
            .iter()
            .position(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError(format!("Clip {} not found", self.clip_id)))?;

        let orig = &track.clips[clip_idx];
        if !orig.timeline_range.contains(self.split_time)
            || self.split_time == orig.timeline_range.start
        {
            return Err(IrisError::TimelineError(
                "Split time is outside clip range or at boundary".into(),
            ));
        }

        let delta_to_split = self.split_time - orig.timeline_range.start;
        let first_duration = delta_to_split;
        let second_duration = orig.timeline_range.duration - delta_to_split;

        let mut clip_b = orig.clone();
        clip_b.id = self.new_clip_id.clone();
        clip_b.source_range =
            TimeSpan::new(orig.source_range.start + delta_to_split, second_duration);
        clip_b.timeline_range = TimeSpan::new(self.split_time, second_duration);

        // Update first clip
        let clip_a = &mut track.clips[clip_idx];
        clip_a.source_range = TimeSpan::new(clip_a.source_range.start, first_duration);
        clip_a.timeline_range = TimeSpan::new(clip_a.timeline_range.start, first_duration);

        track.clips.insert(clip_idx + 1, clip_b);
        track.sort_clips();

        Ok(())
    }

    fn undo(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| {
                IrisError::TimelineError(format!("Track {} not found", self.track_id))
            })?;

        let clip_b_idx = track
            .clips
            .iter()
            .position(|c| c.id == self.new_clip_id)
            .ok_or_else(|| IrisError::TimelineError("Split right-clip not found".into()))?;

        let clip_b = track.clips.remove(clip_b_idx);

        let clip_a = track
            .clips
            .iter_mut()
            .find(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError("Split left-clip not found".into()))?;

        let restored_duration = clip_a.timeline_range.duration + clip_b.timeline_range.duration;
        clip_a.timeline_range.duration = restored_duration;
        clip_a.source_range.duration = restored_duration;

        Ok(())
    }

    fn description(&self) -> String {
        format!("Split clip {} at {}", self.clip_id, self.split_time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimClipCommand {
    pub track_id: String,
    pub clip_id: String,
    pub delta_in: RationalTime,
    pub delta_out: RationalTime,
}

impl Command for TrimClipCommand {
    fn execute(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| IrisError::TimelineError("Track not found".into()))?;

        let clip = track
            .clips
            .iter_mut()
            .find(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError("Clip not found".into()))?;

        clip.source_range.start += self.delta_in;
        clip.timeline_range.start += self.delta_in;
        let new_dur = clip.timeline_range.duration - self.delta_in + self.delta_out;
        if new_dur.numerator <= 0 {
            return Err(IrisError::TimelineError(
                "Trim result produces zero or negative duration".into(),
            ));
        }
        clip.source_range.duration = new_dur;
        clip.timeline_range.duration = new_dur;

        track.sort_clips();
        Ok(())
    }

    fn undo(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| IrisError::TimelineError("Track not found".into()))?;

        let clip = track
            .clips
            .iter_mut()
            .find(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError("Clip not found".into()))?;

        clip.source_range.start -= self.delta_in;
        clip.timeline_range.start -= self.delta_in;
        let old_dur = clip.timeline_range.duration + self.delta_in - self.delta_out;
        clip.source_range.duration = old_dur;
        clip.timeline_range.duration = old_dur;

        track.sort_clips();
        Ok(())
    }

    fn description(&self) -> String {
        format!("Trim clip {}", self.clip_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveClipCommand {
    pub track_id: String,
    pub clip_id: String,
    pub old_start: RationalTime,
    pub new_start: RationalTime,
}

impl Command for MoveClipCommand {
    fn execute(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| IrisError::TimelineError("Track not found".into()))?;

        let clip = track
            .clips
            .iter_mut()
            .find(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError("Clip not found".into()))?;

        clip.timeline_range.start = self.new_start;
        track.sort_clips();
        Ok(())
    }

    fn undo(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| IrisError::TimelineError("Track not found".into()))?;

        let clip = track
            .clips
            .iter_mut()
            .find(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError("Clip not found".into()))?;

        clip.timeline_range.start = self.old_start;
        track.sort_clips();
        Ok(())
    }

    fn description(&self) -> String {
        format!("Move clip {} to {}", self.clip_id, self.new_start)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleDeleteCommand {
    pub track_id: String,
    pub clip_id: String,
    pub deleted_clip: Option<Clip>,
    pub deleted_idx: usize,
}

impl Command for RippleDeleteCommand {
    fn execute(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| IrisError::TimelineError("Track not found".into()))?;

        let idx = track
            .clips
            .iter()
            .position(|c| c.id == self.clip_id)
            .ok_or_else(|| IrisError::TimelineError("Clip not found".into()))?;

        let removed = track.clips.remove(idx);
        let gap = removed.timeline_range.duration;

        // Shift subsequent clips left by gap
        for clip in &mut track.clips[idx..] {
            clip.timeline_range.start -= gap;
        }

        Ok(())
    }

    fn undo(&self, timeline: &mut Timeline) -> IrisResult<()> {
        let clip_to_restore = self
            .deleted_clip
            .as_ref()
            .ok_or_else(|| IrisError::TimelineError("Missing deleted clip data for undo".into()))?;

        let track = timeline
            .tracks
            .iter_mut()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| IrisError::TimelineError("Track not found".into()))?;

        let gap = clip_to_restore.timeline_range.duration;

        // Shift existing clips back right by gap
        for clip in &mut track.clips[self.deleted_idx..] {
            clip.timeline_range.start += gap;
        }

        track
            .clips
            .insert(self.deleted_idx, clip_to_restore.clone());
        track.sort_clips();
        Ok(())
    }

    fn description(&self) -> String {
        format!("Ripple delete clip {}", self.clip_id)
    }
}
