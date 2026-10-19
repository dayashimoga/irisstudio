use crate::source::SourceResolver;
use iris_core::{Dimensions, IrisResult, PixelFormat, RationalTime};
use iris_effects::{blend_pixel, BlendMode};
use iris_media::VideoFrame;
use iris_timeline::{Timeline, TimelineEvaluator};
use rayon::prelude::*;

pub struct Compositor {
    pub canvas_dimensions: Dimensions,
}

impl Compositor {
    pub fn new(canvas_dimensions: Dimensions) -> Self {
        Self { canvas_dimensions }
    }

    pub fn render_frame(
        &self,
        timeline: &Timeline,
        time: RationalTime,
        source_resolver: &dyn SourceResolver,
    ) -> IrisResult<VideoFrame> {
        let active_clips = TimelineEvaluator::evaluate_at(timeline, time);

        let duration = RationalTime::new(timeline.fps_den as i64, timeline.fps_num);
        let mut final_frame =
            VideoFrame::new(self.canvas_dimensions, PixelFormat::Rgba8, time, duration);

        // Fill background with opaque black [0, 0, 0, 255]
        for chunk in final_frame.data.chunks_exact_mut(4) {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            chunk[3] = 255;
        }

        let mut handled_clips = std::collections::HashSet::new();

        // 1. Check for active transitions between clips
        for transition in &timeline.transitions {
            if let Some(progress) = transition.progress_at(time) {
                let clip_a = timeline
                    .tracks
                    .iter()
                    .flat_map(|t| &t.clips)
                    .find(|c| c.id == transition.clip_a_id);
                let clip_b = timeline
                    .tracks
                    .iter()
                    .flat_map(|t| &t.clips)
                    .find(|c| c.id == transition.clip_b_id);

                if let (Some(a), Some(b)) = (clip_a, clip_b) {
                    handled_clips.insert(a.id.clone());
                    handled_clips.insert(b.id.clone());

                    let offset_a = time - a.timeline_range.start;
                    let scaled_num_a = (offset_a.numerator as f64 * a.speed as f64).round() as i64;
                    let src_time_a = a.source_range.start
                        + RationalTime::new(scaled_num_a, offset_a.denominator);

                    let offset_b = time - b.timeline_range.start;
                    let scaled_num_b = (offset_b.numerator as f64 * b.speed as f64).round() as i64;
                    let src_time_b = b.source_range.start
                        + RationalTime::new(scaled_num_b, offset_b.denominator);

                    let frame_a = source_resolver.resolve_video_frame(
                        &a.asset_id,
                        src_time_a,
                        self.canvas_dimensions,
                    )?;
                    let frame_b = source_resolver.resolve_video_frame(
                        &b.asset_id,
                        src_time_b,
                        self.canvas_dimensions,
                    )?;

                    let mut trans_frame =
                        VideoFrame::new(self.canvas_dimensions, PixelFormat::Rgba8, time, duration);
                    crate::transition::render_transition(
                        &frame_a,
                        &frame_b,
                        transition.kind,
                        progress,
                        &mut trans_frame,
                    )?;

                    self.blend_layer(
                        &mut final_frame,
                        &trans_frame,
                        &iris_timeline::ClipTransform::default(),
                        1.0,
                    );
                }
            }
        }

        // 2. Composite remaining active clips from bottom to top
        for clip_state in active_clips {
            if handled_clips.contains(&clip_state.clip_id) {
                continue;
            }

            let source_frame = source_resolver.resolve_video_frame(
                &clip_state.asset_id,
                clip_state.source_time,
                self.canvas_dimensions,
            )?;

            self.blend_layer(
                &mut final_frame,
                &source_frame,
                &clip_state.transform,
                clip_state.opacity,
            );
        }

        // 3. Render subtitle overlays if present
        if !timeline.subtitle_tracks.is_empty() {
            crate::subtitles::render_subtitles_on_frame(
                &mut final_frame,
                &timeline.subtitle_tracks,
                time,
            )?;
        }

        Ok(final_frame)
    }

    fn blend_layer(
        &self,
        dst: &mut VideoFrame,
        src: &VideoFrame,
        transform: &iris_timeline::ClipTransform,
        opacity: f32,
    ) {
        let cw = self.canvas_dimensions.width as f32;
        let ch = self.canvas_dimensions.height as f32;
        let sw = src.dimensions.width as f32;
        let sh = src.dimensions.height as f32;

        let scale_x = transform.scale_x.max(0.001);
        let scale_y = transform.scale_y.max(0.001);

        let rad = -transform.rotation_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        let dst_width = self.canvas_dimensions.width as usize;

        // Render destination canvas pixels in parallel
        dst.data
            .par_chunks_mut(dst_width * 4)
            .enumerate()
            .for_each(|(y, row)| {
                let dy = y as f32 - (ch / 2.0 + transform.position.y);

                for x in 0..dst_width {
                    let dx = x as f32 - (cw / 2.0 + transform.position.x);

                    // Inverse rotation & scale
                    let rot_x = dx * cos_a - dy * sin_a;
                    let rot_y = dx * sin_a + dy * cos_a;

                    let src_x = rot_x / scale_x + sw / 2.0;
                    let src_y = rot_y / scale_y + sh / 2.0;

                    if src_x >= 0.0 && src_x < sw && src_y >= 0.0 && src_y < sh {
                        let sx = (src_x.floor() as usize).min(src.dimensions.width as usize - 1);
                        let sy = (src_y.floor() as usize).min(src.dimensions.height as usize - 1);

                        let src_offset = (sy * src.dimensions.width as usize + sx) * 4;
                        let src_col = [
                            src.data[src_offset],
                            src.data[src_offset + 1],
                            src.data[src_offset + 2],
                            src.data[src_offset + 3],
                        ];

                        let dst_offset = x * 4;
                        let dst_col = [
                            row[dst_offset],
                            row[dst_offset + 1],
                            row[dst_offset + 2],
                            row[dst_offset + 3],
                        ];

                        let blended = blend_pixel(dst_col, src_col, BlendMode::Normal, opacity);
                        row[dst_offset..dst_offset + 4].copy_from_slice(&blended);
                    }
                }
            });
    }
}
