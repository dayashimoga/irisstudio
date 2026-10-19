use crate::validate::validate_exported_media;
use iris_core::{IrisError, IrisResult, Rect};
use iris_effects::{apply_adjustments, AdjustmentParameters};
use iris_images::{
    apply_exif_orientation, crop_image, decode_image, encode_jpeg, encode_png, encode_webp,
    resize_bilinear,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhotoExportFormat {
    Png,
    Jpeg { quality: u8 },
    WebP,
}

impl PhotoExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg { .. } => "jpg",
            Self::WebP => "webp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionStrategy {
    Overwrite,
    RenameWithIndex,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchPhotoItem {
    pub input_path: String,
    pub custom_output_name: Option<String>,
    pub adjustments: Option<AdjustmentParameters>,
    pub crop: Option<Rect>,
    pub resize: Option<(u32, u32)>,
    pub exif_orientation: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchPhotoTask {
    pub items: Vec<BatchPhotoItem>,
    pub destination_dir: String,
    pub naming_template: String, // e.g. "{name}_{preset}_{index}"
    pub preset_name: String,
    pub format: PhotoExportFormat,
    pub collision_strategy: CollisionStrategy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchProgress {
    pub current_index: usize,
    pub total_items: usize,
    pub current_file: String,
    pub success_count: usize,
    pub error_count: usize,
    pub is_complete: bool,
    pub is_cancelled: bool,
    pub message: String,
}

pub struct BatchPhotoProcessor {
    cancel_flag: Arc<AtomicBool>,
}

impl Default for BatchPhotoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchPhotoProcessor {
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

    pub fn process_batch<F>(&self, task: &BatchPhotoTask, mut progress_cb: F) -> IrisResult<()>
    where
        F: FnMut(BatchProgress),
    {
        self.cancel_flag.store(false, Ordering::SeqCst);
        let dest_dir = Path::new(&task.destination_dir);
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir).map_err(|e| {
                IrisError::IoError(format!("Failed to create destination dir: {}", e))
            })?;
        }

        let total = task.items.len();
        let mut success = 0;
        let mut errors = 0;

        for (idx, item) in task.items.iter().enumerate() {
            if self.cancel_flag.load(Ordering::SeqCst) {
                progress_cb(BatchProgress {
                    current_index: idx,
                    total_items: total,
                    current_file: item.input_path.clone(),
                    success_count: success,
                    error_count: errors,
                    is_complete: false,
                    is_cancelled: true,
                    message: "Batch export cancelled by user".into(),
                });
                return Err(IrisError::Cancelled("Batch export cancelled".into()));
            }

            let in_path = Path::new(&item.input_path);
            let stem = in_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| format!("image_{}", idx + 1));

            progress_cb(BatchProgress {
                current_index: idx,
                total_items: total,
                current_file: item.input_path.clone(),
                success_count: success,
                error_count: errors,
                is_complete: false,
                is_cancelled: false,
                message: format!("Processing {} of {}: {}", idx + 1, total, stem),
            });

            match self.process_single_item(item, task, &stem, idx + 1) {
                Ok(out_path) => {
                    // Validate output
                    if let Err(e) = validate_exported_media(&out_path) {
                        errors += 1;
                        let _ = fs::remove_file(&out_path);
                        eprintln!("Batch validation failed for {:?}: {}", out_path, e);
                    } else {
                        success += 1;
                    }
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("Batch item failed for {:?}: {}", item.input_path, e);
                }
            }
        }

        progress_cb(BatchProgress {
            current_index: total,
            total_items: total,
            current_file: String::new(),
            success_count: success,
            error_count: errors,
            is_complete: true,
            is_cancelled: false,
            message: format!(
                "Batch processing complete: {} succeeded, {} failed",
                success, errors
            ),
        });

        Ok(())
    }

    fn process_single_item(
        &self,
        item: &BatchPhotoItem,
        task: &BatchPhotoTask,
        stem: &str,
        index: usize,
    ) -> IrisResult<PathBuf> {
        let input_bytes = fs::read(&item.input_path)
            .map_err(|e| IrisError::IoError(format!("Failed to read input photo: {}", e)))?;

        let mut img = decode_image(&input_bytes)?;

        // 1. EXIF orientation normalization
        if let Some(orientation) = item.exif_orientation {
            img = apply_exif_orientation(&img, orientation);
        }

        // 2. Crop
        if let Some(crop_rect) = item.crop {
            img = crop_image(&img, crop_rect)?;
        }

        // 3. Resize
        if let Some((w, h)) = item.resize {
            img = resize_bilinear(&img, w, h);
        }

        // 4. Adjustments
        if let Some(ref adj) = item.adjustments {
            apply_adjustments(&mut img.pixels, adj);
        }

        // Determine destination file name using template
        let ext = task.format.extension();
        let target_name = if let Some(ref custom) = item.custom_output_name {
            format!("{}.{}", custom, ext)
        } else {
            let rendered = task
                .naming_template
                .replace("{name}", stem)
                .replace("{index}", &format!("{:03}", index))
                .replace("{preset}", &task.preset_name)
                .replace("{ext}", ext);
            format!("{}.{}", rendered, ext)
        };

        let dest_dir = Path::new(&task.destination_dir);
        let mut final_path = dest_dir.join(&target_name);

        // Safeguard: NEVER overwrite original source media
        if final_path == Path::new(&item.input_path) {
            final_path = dest_dir.join(format!("{}_export.{}", stem, ext));
        }

        // Collision resolution
        if final_path.exists() {
            match task.collision_strategy {
                CollisionStrategy::Overwrite => {
                    // Allowed
                }
                CollisionStrategy::RenameWithIndex => {
                    let mut counter = 1;
                    loop {
                        let renamed = dest_dir.join(format!("{}_{}.{}", stem, counter, ext));
                        if !renamed.exists() {
                            final_path = renamed;
                            break;
                        }
                        counter += 1;
                    }
                }
                CollisionStrategy::Skip => {
                    return Ok(final_path); // Skip write
                }
            }
        }

        // Encode image
        let encoded_bytes = match task.format {
            PhotoExportFormat::Png => encode_png(&img)?,
            PhotoExportFormat::Jpeg { quality } => encode_jpeg(&img, quality)?,
            PhotoExportFormat::WebP => encode_webp(&img)?,
        };

        // Atomic publish via temp file
        let temp_path = final_path.with_extension("tmp");
        fs::write(&temp_path, &encoded_bytes)
            .map_err(|e| IrisError::IoError(format!("Failed to write temp export: {}", e)))?;

        fs::rename(&temp_path, &final_path)
            .map_err(|e| IrisError::IoError(format!("Failed to commit batch export: {}", e)))?;

        Ok(final_path)
    }
}
