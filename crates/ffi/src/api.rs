#![allow(clippy::not_unsafe_ptr_arg_deref)]

use iris_audio::{generate_waveform_peaks, WaveformPeak};
use iris_cache::{CacheManager, MemoryBudgetTier};
use iris_core::{Dimensions, Point, RationalTime, Rect};
use iris_effects::{
    apply_adjustments, apply_brush_stroke, apply_clone_stamp, apply_denoise, apply_gaussian_blur,
    apply_heal_brush, apply_levels, apply_red_eye_removal, apply_unsharp_mask, apply_vignette,
    AdjustmentParameters, LevelsParameters,
};
use iris_export::{validate_exported_media, BatchPhotoProcessor, BatchPhotoTask};
use iris_images::{
    apply_exif_orientation, apply_lens_correction, apply_perspective_warp, crop_image,
    decode_image, encode_jpeg, encode_png, encode_webp, flip_horizontal, flip_vertical,
    resize_bilinear, rotate_90, straighten_image, RgbaImage,
};
use iris_projects::{CrashJournal, Project, UndoRedoManager};
use iris_render::{MockSourceResolver, PreviewEngine};
use iris_timeline::{Clip, RippleDeleteCommand, SplitClipCommand, Track, TrackKind};
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct IrisEngine {
    pub project: Project,
    pub undo_manager: UndoRedoManager,
    pub preview_engine: PreviewEngine,
    pub source_resolver: MockSourceResolver,
    pub cache: CacheManager,
}

static ENGINE_SEQ: AtomicU64 = AtomicU64::new(1);

#[no_mangle]
pub extern "C" fn iris_engine_version() -> *mut c_char {
    let s = CString::new("Iris Studio 0.1.0-alpha (Rust Stable Engine)").unwrap();
    s.into_raw()
}

#[no_mangle]
pub extern "C" fn iris_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn iris_engine_create(name: *const c_char) -> *mut IrisEngine {
    let project_name = if name.is_null() {
        "Untitled Project".to_string()
    } else {
        unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() }
    };

    let id = format!("prj_{}", ENGINE_SEQ.fetch_add(1, Ordering::SeqCst));
    let mut project = Project::new(id, project_name);

    // Initialize default starter tracks
    let mut v_track = Track::new("trk_v1", "Video 1", TrackKind::Video);
    let clip1 = Clip::new(
        "clip_1",
        "test_video_1",
        "Opening Scene",
        iris_core::TimeSpan::new(RationalTime::ZERO, RationalTime::new(5, 1)),
        RationalTime::ZERO,
    );
    let clip2 = Clip::new(
        "clip_2",
        "test_video_2",
        "Feature Presentation",
        iris_core::TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)),
        RationalTime::new(5, 1),
    );
    v_track.clips.push(clip1);
    v_track.clips.push(clip2);

    let a_track = Track::new("trk_a1", "Audio 1 (Master)", TrackKind::Audio);

    project.timeline.tracks.push(v_track);
    project.timeline.tracks.push(a_track);

    let engine = IrisEngine {
        project,
        undo_manager: UndoRedoManager::new(100),
        preview_engine: PreviewEngine::new(Dimensions::new(1920, 1080)),
        source_resolver: MockSourceResolver::default(),
        cache: CacheManager::new(MemoryBudgetTier::DesktopStandard),
    };

    Box::into_raw(Box::new(engine))
}

#[no_mangle]
pub extern "C" fn iris_engine_free(ptr: *mut IrisEngine) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_adjustments(
    width: u32,
    height: u32,
    pixels: *mut u8,
    params_json: *const c_char,
) -> c_int {
    if pixels.is_null() || params_json.is_null() {
        return -1;
    }

    let json_str = unsafe { CStr::from_ptr(params_json).to_string_lossy() };
    let params: AdjustmentParameters = match serde_json::from_str(&json_str) {
        Ok(p) => p,
        Err(_) => return -2,
    };

    let total_bytes = (width as usize) * (height as usize) * 4;
    let pixel_slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };

    apply_adjustments(pixel_slice, &params);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_crop(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    x: f32,
    y: f32,
    crop_w: f32,
    crop_h: f32,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null() || dst_pixels.is_null() {
        return -1;
    }

    let src_slice = unsafe { slice::from_raw_parts(src_pixels, (width * height * 4) as usize) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };

    let rect = Rect::new(x, y, crop_w, crop_h);
    let cropped = match crop_image(&img, rect) {
        Ok(c) => c,
        Err(_) => return -3,
    };

    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, cropped.pixels.len()) };
    dst_slice.copy_from_slice(&cropped.pixels);

    0
}

#[no_mangle]
pub extern "C" fn iris_timeline_render_preview(
    engine_ptr: *mut IrisEngine,
    time_num: i64,
    time_den: u64,
    out_pixels: *mut u8,
    out_max_bytes: usize,
) -> c_int {
    if engine_ptr.is_null() || out_pixels.is_null() {
        return -1;
    }

    let engine = unsafe { &mut *engine_ptr };
    let time = RationalTime::new(time_num, time_den);

    match engine.preview_engine.render_preview(
        &engine.project.timeline,
        time,
        &engine.source_resolver,
    ) {
        Ok(Some(frame)) => {
            if frame.data.len() > out_max_bytes {
                return -2;
            }
            let dst_slice = unsafe { slice::from_raw_parts_mut(out_pixels, frame.data.len()) };
            dst_slice.copy_from_slice(&frame.data);
            0
        }
        Ok(None) => 1, // Frame dropped due to backpressure / superseded
        Err(_) => -3,
    }
}

#[no_mangle]
pub extern "C" fn iris_timeline_split_clip(
    engine_ptr: *mut IrisEngine,
    track_id: *const c_char,
    clip_id: *const c_char,
    split_num: i64,
    split_den: u64,
) -> c_int {
    if engine_ptr.is_null() || track_id.is_null() || clip_id.is_null() {
        return -1;
    }

    let engine = unsafe { &mut *engine_ptr };
    let trk = unsafe { CStr::from_ptr(track_id).to_string_lossy().into_owned() };
    let clp = unsafe { CStr::from_ptr(clip_id).to_string_lossy().into_owned() };
    let split_time = RationalTime::new(split_num, split_den);
    let new_clip_id = format!("{}_part2", clp);

    let cmd = SplitClipCommand {
        track_id: trk,
        clip_id: clp,
        split_time,
        new_clip_id,
    };

    match engine
        .undo_manager
        .execute_command(Box::new(cmd), &mut engine.project.timeline)
    {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn iris_timeline_ripple_delete(
    engine_ptr: *mut IrisEngine,
    track_id: *const c_char,
    clip_id: *const c_char,
) -> c_int {
    if engine_ptr.is_null() || track_id.is_null() || clip_id.is_null() {
        return -1;
    }

    let engine = unsafe { &mut *engine_ptr };
    let trk = unsafe { CStr::from_ptr(track_id).to_string_lossy().into_owned() };
    let clp = unsafe { CStr::from_ptr(clip_id).to_string_lossy().into_owned() };

    let track = match engine.project.timeline.tracks.iter().find(|t| t.id == trk) {
        Some(t) => t,
        None => return -2,
    };

    let idx = match track.clips.iter().position(|c| c.id == clp) {
        Some(i) => i,
        None => return -3,
    };

    let deleted_clip = track.clips[idx].clone();

    let cmd = RippleDeleteCommand {
        track_id: trk,
        clip_id: clp,
        deleted_clip: Some(deleted_clip),
        deleted_idx: idx,
    };

    match engine
        .undo_manager
        .execute_command(Box::new(cmd), &mut engine.project.timeline)
    {
        Ok(_) => 0,
        Err(_) => -4,
    }
}

#[no_mangle]
pub extern "C" fn iris_undo(engine_ptr: *mut IrisEngine) -> c_int {
    if engine_ptr.is_null() {
        return -1;
    }
    let engine = unsafe { &mut *engine_ptr };
    match engine.undo_manager.undo(&mut engine.project.timeline) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn iris_redo(engine_ptr: *mut IrisEngine) -> c_int {
    if engine_ptr.is_null() {
        return -1;
    }
    let engine = unsafe { &mut *engine_ptr };
    match engine.undo_manager.redo(&mut engine.project.timeline) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn iris_get_audio_waveform(
    samples_ptr: *const f32,
    sample_count: usize,
    channels: u16,
    samples_per_pixel: usize,
    out_peaks_ptr: *mut WaveformPeak,
    max_peaks: usize,
) -> c_int {
    if samples_ptr.is_null() || out_peaks_ptr.is_null() || sample_count == 0 || channels == 0 {
        return -1;
    }

    let samples = unsafe { slice::from_raw_parts(samples_ptr, sample_count) };
    let envelope = generate_waveform_peaks(samples, channels, samples_per_pixel);

    let count = envelope.peaks.len().min(max_peaks);
    let dst_slice = unsafe { slice::from_raw_parts_mut(out_peaks_ptr, count) };
    dst_slice.copy_from_slice(&envelope.peaks[..count]);

    count as c_int
}

#[no_mangle]
pub extern "C" fn iris_project_get_json(engine_ptr: *mut IrisEngine) -> *mut c_char {
    if engine_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let engine = unsafe { &*engine_ptr };
    match engine.project.to_json() {
        Ok(json) => match CString::new(json) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn iris_crash_journal_has_recovery(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }
    let p = unsafe { CStr::from_ptr(path).to_string_lossy() };
    let journal = CrashJournal::new(p.as_ref());
    if journal.has_unrecovered_crash() {
        1
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Phase 1: Photo Processing C-ABI Surface
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn iris_photo_decode_file(
    file_path: *const c_char,
    out_width: *mut u32,
    out_height: *mut u32,
    out_pixels: *mut *mut u8,
) -> c_int {
    if file_path.is_null() || out_width.is_null() || out_height.is_null() || out_pixels.is_null() {
        return -1;
    }

    let path_str = unsafe { CStr::from_ptr(file_path).to_string_lossy() };
    let bytes = match fs::read(path_str.as_ref()) {
        Ok(b) => b,
        Err(_) => return -2,
    };

    let img = match decode_image(&bytes) {
        Ok(i) => i,
        Err(_) => return -3,
    };

    unsafe {
        *out_width = img.width;
        *out_height = img.height;
        let mut boxed_slice = img.pixels.into_boxed_slice();
        *out_pixels = boxed_slice.as_mut_ptr();
        std::mem::forget(boxed_slice);
    }

    0
}

#[no_mangle]
pub extern "C" fn iris_photo_free_pixels(pixels: *mut u8, total_bytes: usize) {
    if !pixels.is_null() && total_bytes > 0 {
        unsafe {
            let slice = slice::from_raw_parts_mut(pixels, total_bytes);
            drop(Box::from_raw(slice));
        }
    }
}

#[no_mangle]
pub extern "C" fn iris_photo_encode_file(
    output_path: *const c_char,
    format_code: c_int, // 0 = PNG, 1 = JPEG, 2 = WebP
    quality: u8,
    width: u32,
    height: u32,
    pixels: *const u8,
) -> c_int {
    if output_path.is_null() || pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let out_str = unsafe { CStr::from_ptr(output_path).to_string_lossy() };
    let pixel_len = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts(pixels, pixel_len) };

    let img = match RgbaImage::from_raw(width, height, slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };

    let encoded_bytes = match format_code {
        0 => match encode_png(&img) {
            Ok(b) => b,
            Err(_) => return -3,
        },
        1 => match encode_jpeg(&img, quality) {
            Ok(b) => b,
            Err(_) => return -3,
        },
        2 => match encode_webp(&img) {
            Ok(b) => b,
            Err(_) => return -3,
        },
        _ => return -4,
    };

    let path = Path::new(out_str.as_ref());
    let temp_path = path.with_extension("tmp");
    if fs::write(&temp_path, &encoded_bytes).is_err() {
        return -5;
    }

    if fs::rename(&temp_path, path).is_err() {
        return -6;
    }

    if validate_exported_media(path).is_err() {
        let _ = fs::remove_file(path);
        return -7;
    }

    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_levels(
    width: u32,
    height: u32,
    pixels: *mut u8,
    in_black: f32,
    in_gamma: f32,
    in_white: f32,
    out_black: f32,
    out_white: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    let params = LevelsParameters {
        in_black,
        in_gamma,
        in_white,
        out_black,
        out_white,
    };

    apply_levels(slice, &params);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_denoise(
    width: u32,
    height: u32,
    pixels: *mut u8,
    radius: i32,
    strength: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_denoise(slice, width, height, radius, strength);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_red_eye(
    width: u32,
    height: u32,
    pixels: *mut u8,
    center_x: u32,
    center_y: u32,
    radius: f32,
    threshold: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_red_eye_removal(slice, width, height, center_x, center_y, radius, threshold);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_clone_stamp(
    width: u32,
    height: u32,
    pixels: *mut u8,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    radius: f32,
    hardness: f32,
    opacity: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_clone_stamp(
        slice, width, height, src_x, src_y, dst_x, dst_y, radius, hardness, opacity,
    );
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_heal_brush(
    width: u32,
    height: u32,
    pixels: *mut u8,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    radius: f32,
    hardness: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_heal_brush(
        slice, width, height, src_x, src_y, dst_x, dst_y, radius, hardness,
    );
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_lens_correction(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    k1: f32,
    k2: f32,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null() || dst_pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, total_bytes) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };

    let corrected = apply_lens_correction(&img, k1, k2);
    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, total_bytes) };
    dst_slice.copy_from_slice(&corrected.pixels);

    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_perspective(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    p0x: f32,
    p0y: f32,
    p1x: f32,
    p1y: f32,
    p2x: f32,
    p2y: f32,
    p3x: f32,
    p3y: f32,
    dst_w: u32,
    dst_h: u32,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null()
        || dst_pixels.is_null()
        || width == 0
        || height == 0
        || dst_w == 0
        || dst_h == 0
    {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, total_bytes) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };

    let quad = [
        Point::new(p0x, p0y),
        Point::new(p1x, p1y),
        Point::new(p2x, p2y),
        Point::new(p3x, p3y),
    ];

    let warped = match apply_perspective_warp(&img, quad, dst_w, dst_h) {
        Ok(w) => w,
        Err(_) => return -3,
    };

    let dst_len = (dst_w * dst_h * 4) as usize;
    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, dst_len) };
    dst_slice.copy_from_slice(&warped.pixels);

    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_exif_orientation(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    orientation: u32,
    out_w: *mut u32,
    out_h: *mut u32,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null() || dst_pixels.is_null() || out_w.is_null() || out_h.is_null() {
        return -1;
    }

    let total_bytes = (width * height * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, total_bytes) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };

    let oriented = apply_exif_orientation(&img, orientation);
    unsafe {
        *out_w = oriented.width;
        *out_h = oriented.height;
    }

    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, oriented.pixels.len()) };
    dst_slice.copy_from_slice(&oriented.pixels);

    0
}

#[no_mangle]
pub extern "C" fn iris_batch_photo_run(task_json: *const c_char) -> *mut c_char {
    if task_json.is_null() {
        return std::ptr::null_mut();
    }

    let json_str = unsafe { CStr::from_ptr(task_json).to_string_lossy() };
    let task: BatchPhotoTask = match serde_json::from_str(&json_str) {
        Ok(t) => t,
        Err(e) => {
            let err_json = format!(r#"{{"success":false,"error":"Invalid JSON: {}"}}"#, e);
            return CString::new(err_json).unwrap().into_raw();
        }
    };

    let processor = BatchPhotoProcessor::new();
    let mut last_progress: Option<iris_export::BatchProgress> = None;

    let res = processor.process_batch(&task, |progress| {
        last_progress = Some(progress);
    });

    let summary = match res {
        Ok(_) => {
            let p = last_progress.unwrap_or(iris_export::BatchProgress {
                current_index: task.items.len(),
                total_items: task.items.len(),
                current_file: String::new(),
                success_count: task.items.len(),
                error_count: 0,
                is_complete: true,
                is_cancelled: false,
                message: "Completed successfully".into(),
            });
            format!(
                r#"{{"success":true,"total":{},"succeeded":{},"failed":{}}}"#,
                p.total_items, p.success_count, p.error_count
            )
        }
        Err(e) => format!(r#"{{"success":false,"error":"{}"}}"#, e),
    };

    CString::new(summary).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_vignette(
    width: u32,
    height: u32,
    pixels: *mut u8,
    amount: f32,
    midpoint: f32,
    feather: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_vignette(slice, width, height, amount, midpoint, feather);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_unsharp_mask(
    width: u32,
    height: u32,
    pixels: *mut u8,
    amount: f32,
    radius: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_unsharp_mask(slice, width, height, amount, radius);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_blur(
    width: u32,
    height: u32,
    pixels: *mut u8,
    radius: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_gaussian_blur(slice, width, height, radius);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_straighten(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    angle_degrees: f32,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null() || dst_pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let total_bytes = (width * height * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, total_bytes) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };
    let straightened = straighten_image(&img, angle_degrees);
    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, total_bytes) };
    dst_slice.copy_from_slice(&straightened.pixels);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_rotate_90(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    clockwise: bool,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null() || dst_pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let total_bytes = (width * height * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, total_bytes) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };
    let rotated = rotate_90(&img, clockwise);
    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, total_bytes) };
    dst_slice.copy_from_slice(&rotated.pixels);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_flip(
    width: u32,
    height: u32,
    src_pixels: *const u8,
    horizontal: bool,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null() || dst_pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let total_bytes = (width * height * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, total_bytes) };
    let img = match RgbaImage::from_raw(width, height, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };
    let flipped = if horizontal {
        flip_horizontal(&img)
    } else {
        flip_vertical(&img)
    };
    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, total_bytes) };
    dst_slice.copy_from_slice(&flipped.pixels);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_resize(
    src_w: u32,
    src_h: u32,
    src_pixels: *const u8,
    new_w: u32,
    new_h: u32,
    dst_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null()
        || dst_pixels.is_null()
        || src_w == 0
        || src_h == 0
        || new_w == 0
        || new_h == 0
    {
        return -1;
    }
    let src_bytes = (src_w * src_h * 4) as usize;
    let src_slice = unsafe { slice::from_raw_parts(src_pixels, src_bytes) };
    let img = match RgbaImage::from_raw(src_w, src_h, src_slice.to_vec()) {
        Ok(i) => i,
        Err(_) => return -2,
    };
    let resized = resize_bilinear(&img, new_w, new_h);
    let dst_bytes = (new_w * new_h * 4) as usize;
    let dst_slice = unsafe { slice::from_raw_parts_mut(dst_pixels, dst_bytes) };
    dst_slice.copy_from_slice(&resized.pixels);
    0
}

#[no_mangle]
pub extern "C" fn iris_photo_apply_brush(
    width: u32,
    height: u32,
    pixels: *mut u8,
    points_coords: *const f32, // Array of pairs [x0, y0, x1, y1, ...]
    num_points: usize,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    radius: f32,
    hardness: f32,
    erase: bool,
) -> c_int {
    if pixels.is_null() || points_coords.is_null() || width == 0 || height == 0 || num_points == 0 {
        return -1;
    }
    let coord_slice = unsafe { slice::from_raw_parts(points_coords, num_points * 2) };
    let mut points = Vec::with_capacity(num_points);
    for chunk in coord_slice.chunks_exact(2) {
        points.push((chunk[0], chunk[1]));
    }
    let total_bytes = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, total_bytes) };
    apply_brush_stroke(
        slice,
        width,
        height,
        &points,
        [r, g, b, a],
        radius,
        hardness,
        erase,
    );
    0
}
