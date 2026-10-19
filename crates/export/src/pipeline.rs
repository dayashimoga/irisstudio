use crate::config::ExportConfig;
use crate::validate::validate_exported_media;
use iris_core::{IrisError, IrisResult, RationalTime};
use iris_render::{Compositor, SourceResolver};
use iris_timeline::Timeline;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportProgress {
    pub current_frame: u64,
    pub total_frames: u64,
    pub fps: f32,
    pub percent: f32,
    pub is_cancelled: bool,
    pub is_complete: bool,
    pub error: Option<String>,
}

pub struct ExportScheduler {
    cancel_flag: Arc<AtomicBool>,
}

impl Default for ExportScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportScheduler {
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel_token(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    pub fn run_export<F>(
        &self,
        timeline: &Timeline,
        config: &ExportConfig,
        source_resolver: &dyn SourceResolver,
        mut progress_cb: F,
    ) -> IrisResult<()>
    where
        F: FnMut(ExportProgress),
    {
        self.cancel_flag.store(false, Ordering::SeqCst);

        let start_time = config.range_in.unwrap_or(RationalTime::ZERO);
        let end_time = config
            .range_out
            .unwrap_or_else(|| timeline.total_duration());

        if end_time <= start_time {
            return Err(IrisError::TimelineError(
                "Export duration must be greater than zero".into(),
            ));
        }

        let duration = end_time - start_time;
        let total_frames = duration.to_frames(config.fps_num, config.fps_den).max(1) as u64;

        let compositor = Compositor::new(config.dimensions);
        let mut out_file = File::create(&config.output_path)
            .map_err(|e| IrisError::IoError(format!("Failed to create export file: {}", e)))?;

        // Write container header identifier
        let header = format!(
            "IRIS_STUDIO_EXPORT_CONTAINER_V1|WIDTH={}|HEIGHT={}|FPS={}/{}|FRAMES={}\n",
            config.dimensions.width,
            config.dimensions.height,
            config.fps_num,
            config.fps_den,
            total_frames
        );
        out_file
            .write_all(header.as_bytes())
            .map_err(|e| IrisError::IoError(e.to_string()))?;

        let bench_start = Instant::now();

        for f in 0..total_frames {
            if self.cancel_flag.load(Ordering::SeqCst) {
                let _ = std::fs::remove_file(&config.output_path);
                progress_cb(ExportProgress {
                    current_frame: f,
                    total_frames,
                    fps: 0.0,
                    percent: (f as f32 / total_frames as f32) * 100.0,
                    is_cancelled: true,
                    is_complete: false,
                    error: Some("Export cancelled by user".into()),
                });
                return Err(IrisError::Cancelled("Export aborted".into()));
            }

            let cur_time =
                start_time + RationalTime::new((f * config.fps_den) as i64, config.fps_num);
            let frame = compositor.render_frame(timeline, cur_time, source_resolver)?;

            // Write packed frame slice
            out_file
                .write_all(&frame.data)
                .map_err(|e| IrisError::IoError(e.to_string()))?;

            let elapsed = bench_start.elapsed().as_secs_f32().max(0.001);
            let cur_fps = (f + 1) as f32 / elapsed;
            let percent = ((f + 1) as f32 / total_frames as f32) * 100.0;

            progress_cb(ExportProgress {
                current_frame: f + 1,
                total_frames,
                fps: cur_fps,
                percent,
                is_cancelled: false,
                is_complete: f + 1 == total_frames,
                error: None,
            });
        }

        out_file
            .flush()
            .map_err(|e| IrisError::IoError(e.to_string()))?;
        drop(out_file);

        // Validation pass
        validate_exported_media(&config.output_path)?;

        Ok(())
    }
}
