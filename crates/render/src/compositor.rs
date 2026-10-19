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

        if active_clips.is_empty() {
            return Ok(final_frame);
        }

        // Composite active clips from bottom to top
        for clip_state in active_clips {
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
