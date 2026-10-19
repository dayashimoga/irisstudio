use crate::compositor::Compositor;
use crate::source::SourceResolver;
use iris_core::{Dimensions, IrisResult, RationalTime};
use iris_media::VideoFrame;
use iris_timeline::Timeline;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct PreviewEngine {
    compositor: Compositor,
    latest_token: Arc<AtomicU64>,
}

impl PreviewEngine {
    pub fn new(preview_dimensions: Dimensions) -> Self {
        Self {
            compositor: Compositor::new(preview_dimensions),
            latest_token: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.compositor.canvas_dimensions
    }

    pub fn resize(&mut self, dimensions: Dimensions) {
        self.compositor.canvas_dimensions = dimensions;
    }

    pub fn render_preview(
        &self,
        timeline: &Timeline,
        time: RationalTime,
        source_resolver: &dyn SourceResolver,
    ) -> IrisResult<Option<VideoFrame>> {
        let token = self.latest_token.fetch_add(1, Ordering::SeqCst) + 1;

        // Render frame
        let frame = self
            .compositor
            .render_frame(timeline, time, source_resolver)?;

        // Backpressure check: if another render was triggered while we rendered, discard obsolete frame
        if self.latest_token.load(Ordering::SeqCst) != token {
            return Ok(None); // Dropped outdated frame
        }

        Ok(Some(frame))
    }
}
