use crate::model::Clip;
use iris_core::{RationalTime, TimeSpan};
use serde::{Deserialize, Serialize};

/// Represents a distinct camera angle / stream in a multi-camera setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraAngle {
    pub id: String,
    pub name: String,
    pub asset_id: String,
    pub sync_offset: RationalTime,
    pub is_audio_source: bool,
    pub color_label: String,
}

impl CameraAngle {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        asset_id: impl Into<String>,
        sync_offset: RationalTime,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            asset_id: asset_id.into(),
            sync_offset,
            is_audio_source: false,
            color_label: "#00D1FF".into(),
        }
    }
}

/// A cut point in the timeline switching to a specific camera angle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AngleCut {
    pub timeline_time: RationalTime,
    pub angle_id: String,
}

/// Multi-camera composite clip containing synchronized angles and live angle cuts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiCamClip {
    pub id: String,
    pub name: String,
    pub timeline_range: TimeSpan,
    pub angles: Vec<CameraAngle>,
    pub cuts: Vec<AngleCut>,
    pub default_angle_id: String,
}

impl MultiCamClip {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        timeline_range: TimeSpan,
        default_angle_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            timeline_range,
            angles: Vec::new(),
            cuts: Vec::new(),
            default_angle_id: default_angle_id.into(),
        }
    }

    /// Adds a camera angle to this multicam group.
    pub fn add_angle(&mut self, angle: CameraAngle) {
        if !self.angles.iter().any(|a| a.id == angle.id) {
            self.angles.push(angle);
        }
    }

    /// Retrieves a camera angle by its unique ID.
    pub fn get_angle(&self, angle_id: &str) -> Option<&CameraAngle> {
        self.angles.iter().find(|a| a.id == angle_id)
    }

    /// Evaluates which camera angle is active at the specified timeline time.
    pub fn active_angle_at(&self, time: RationalTime) -> Option<&CameraAngle> {
        if self.angles.is_empty() {
            return None;
        }

        // Find the latest cut that occurred at or before `time`
        let active_cut = self
            .cuts
            .iter()
            .filter(|cut| cut.timeline_time <= time)
            .max_by_key(|cut| cut.timeline_time);

        let target_id = match active_cut {
            Some(cut) => &cut.angle_id,
            None => &self.default_angle_id,
        };

        self.angles
            .iter()
            .find(|a| a.id == *target_id)
            .or_else(|| self.angles.first())
    }

    /// Switches the active angle at `time`, recording a cut.
    /// If a cut already exists at the exact timestamp, updates its target angle.
    pub fn switch_angle(&mut self, time: RationalTime, angle_id: &str) -> bool {
        if !self.angles.iter().any(|a| a.id == angle_id) {
            return false;
        }

        // Check if a cut exists at the exact time
        if let Some(existing) = self.cuts.iter_mut().find(|c| c.timeline_time == time) {
            existing.angle_id = angle_id.to_string();
        } else {
            self.cuts.push(AngleCut {
                timeline_time: time,
                angle_id: angle_id.to_string(),
            });
            self.cuts.sort_by_key(|c| c.timeline_time);
        }

        true
    }

    /// Calculates the source media timestamp for a given timeline time and angle.
    pub fn source_time_for_angle(&self, time: RationalTime, angle_id: &str) -> RationalTime {
        let offset = self
            .get_angle(angle_id)
            .map(|a| a.sync_offset)
            .unwrap_or(RationalTime::ZERO);
        let rel_time = time - self.timeline_range.start;
        offset + rel_time
    }

    /// Flattens the multicam cuts into standard individual `Clip` objects.
    pub fn flatten_to_clips(&self) -> Vec<Clip> {
        if self.angles.is_empty() {
            return Vec::new();
        }

        let mut cut_points: Vec<RationalTime> = Vec::new();
        cut_points.push(self.timeline_range.start);

        for cut in &self.cuts {
            if cut.timeline_time > self.timeline_range.start
                && cut.timeline_time < self.timeline_range.end()
            {
                cut_points.push(cut.timeline_time);
            }
        }
        cut_points.push(self.timeline_range.end());
        cut_points.dedup();
        cut_points.sort();

        let mut result = Vec::new();
        for i in 0..(cut_points.len() - 1) {
            let start = cut_points[i];
            let end = cut_points[i + 1];
            let duration = end - start;
            if duration <= RationalTime::ZERO {
                continue;
            }

            if let Some(angle) = self.active_angle_at(start) {
                let source_start = self.source_time_for_angle(start, &angle.id);
                let clip_id = format!("{}_cut_{}", self.id, i + 1);
                let clip_name = format!("{} ({})", self.name, angle.name);

                let clip = Clip::new(
                    clip_id,
                    &angle.asset_id,
                    clip_name,
                    TimeSpan::new(source_start, duration),
                    start,
                );
                result.push(clip);
            }
        }

        result
    }
}
