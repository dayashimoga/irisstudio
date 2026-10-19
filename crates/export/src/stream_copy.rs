//! Stream copy optimization for unedited timeline cuts.
//!
//! Analyzes timeline edit decisions to identify regions where video/audio packets
//! can be directly remuxed without decoding and re-encoding, maximizing export
//! speed and preserving 100% original stream bitstream fidelity.

use crate::config::ExportConfig;
use iris_core::{RationalTime, TimeSpan};
use iris_timeline::{Timeline, TrackKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamProcessingMode {
    /// Packet-level direct stream copy (zero re-encoding loss, 30x+ export throughput).
    StreamCopy {
        source_asset_id: String,
        source_range: TimeSpan,
    },
    /// Frame-by-frame rendering and encoding required.
    ReEncode { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamCopySegment {
    pub timeline_range: TimeSpan,
    pub mode: StreamProcessingMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamCopyPlan {
    pub total_duration: RationalTime,
    pub segments: Vec<StreamCopySegment>,
    pub can_full_stream_copy: bool,
    pub stream_copy_percentage: f32,
    pub estimated_speedup: f32,
}

pub struct StreamCopyOptimizer;

impl StreamCopyOptimizer {
    /// Analyzes the timeline against the export configuration to build an optimal
    /// stream-copy remuxing vs re-encoding plan.
    pub fn analyze_timeline(timeline: &Timeline, _config: &ExportConfig) -> StreamCopyPlan {
        let total_duration = timeline.total_duration();
        if total_duration == RationalTime::ZERO {
            return StreamCopyPlan {
                total_duration,
                segments: Vec::new(),
                can_full_stream_copy: false,
                stream_copy_percentage: 0.0,
                estimated_speedup: 1.0,
            };
        }

        let video_tracks: Vec<_> = timeline
            .tracks
            .iter()
            .filter(|t| t.kind == TrackKind::Video && !t.muted)
            .collect();

        // If multiple active video tracks overlap, compositing / re-encoding is necessary
        let has_multitrack_video = video_tracks.len() > 1;

        let mut segments = Vec::new();
        let mut copy_duration_sec = 0.0f64;

        if has_multitrack_video {
            segments.push(StreamCopySegment {
                timeline_range: TimeSpan::new(RationalTime::ZERO, total_duration),
                mode: StreamProcessingMode::ReEncode {
                    reason: "Multiple active video tracks require compositing".to_string(),
                },
            });
        } else if let Some(track) = video_tracks.first() {
            for clip in &track.clips {
                let is_transform_default = clip.transform.position.x == 0.0
                    && clip.transform.position.y == 0.0
                    && (clip.transform.scale_x - 1.0).abs() < 1e-4
                    && (clip.transform.scale_y - 1.0).abs() < 1e-4
                    && clip.transform.rotation_deg == 0.0
                    && (clip.transform.opacity - 1.0).abs() < 1e-4;

                let is_speed_normal = (clip.speed - 1.0).abs() < 1e-4;

                // Check if any transition intersects this clip
                let clip_has_transition = timeline.transitions.iter().any(|tr| {
                    (tr.clip_a_id == clip.id || tr.clip_b_id == clip.id)
                        && clip
                            .timeline_range
                            .intersects(TimeSpan::new(tr.start_time, tr.duration))
                });

                // Check if subtitles overlap this clip
                let has_subtitles = timeline.subtitle_tracks.iter().any(|st| {
                    st.enabled
                        && st.items.iter().any(|item| {
                            item.start_time < clip.timeline_range.end()
                                && item.end_time > clip.timeline_range.start
                        })
                });

                if !is_transform_default {
                    segments.push(StreamCopySegment {
                        timeline_range: clip.timeline_range,
                        mode: StreamProcessingMode::ReEncode {
                            reason: "Clip spatial transform or opacity applied".to_string(),
                        },
                    });
                } else if !is_speed_normal {
                    segments.push(StreamCopySegment {
                        timeline_range: clip.timeline_range,
                        mode: StreamProcessingMode::ReEncode {
                            reason: "Clip playback speed altered".to_string(),
                        },
                    });
                } else if clip_has_transition {
                    segments.push(StreamCopySegment {
                        timeline_range: clip.timeline_range,
                        mode: StreamProcessingMode::ReEncode {
                            reason: "Clip involved in active transition blend".to_string(),
                        },
                    });
                } else if has_subtitles {
                    segments.push(StreamCopySegment {
                        timeline_range: clip.timeline_range,
                        mode: StreamProcessingMode::ReEncode {
                            reason: "Subtitle overlay burn-in active".to_string(),
                        },
                    });
                } else {
                    // Clip is unedited, cut-only! Eligible for zero-loss stream copy!
                    copy_duration_sec += clip.timeline_range.duration.to_seconds();
                    segments.push(StreamCopySegment {
                        timeline_range: clip.timeline_range,
                        mode: StreamProcessingMode::StreamCopy {
                            source_asset_id: clip.asset_id.clone(),
                            source_range: clip.source_range,
                        },
                    });
                }
            }
        } else {
            // No video tracks
            segments.push(StreamCopySegment {
                timeline_range: TimeSpan::new(RationalTime::ZERO, total_duration),
                mode: StreamProcessingMode::ReEncode {
                    reason: "No active video track present".to_string(),
                },
            });
        }

        let total_sec = total_duration.to_seconds().max(1e-6);
        let copy_pct = ((copy_duration_sec / total_sec) * 100.0).clamp(0.0, 100.0) as f32;
        let can_full_copy = (copy_pct - 100.0).abs() < 1e-2 && !segments.is_empty();

        // Re-encoding is ~1.0x baseline, stream copy is ~25x faster
        let estimated_speedup = 1.0 + (copy_pct / 100.0) * 24.0;

        StreamCopyPlan {
            total_duration,
            segments,
            can_full_stream_copy: can_full_copy,
            stream_copy_percentage: copy_pct,
            estimated_speedup,
        }
    }
}
