#![allow(clippy::not_unsafe_ptr_arg_deref)]

use iris_audio::{
    generate_waveform_peaks, AudioCompressor, AudioDucker, BinauralPanner, DuckingParameters,
    EqualizerBand, ThreeBandEq, WaveformPeak, WsolaTimeStretch,
};
use iris_cache::{CacheManager, MemoryBudgetTier};
use iris_core::{Dimensions, PixelFormat, Point, RationalTime, Rect};
use iris_effects::{
    apply_adjustments, apply_brush_stroke, apply_chroma_key, apply_clone_stamp, apply_denoise,
    apply_gaussian_blur, apply_hdr_tonemap, apply_heal_brush, apply_levels, apply_red_eye_removal,
    apply_smart_mask, apply_unsharp_mask, apply_vignette, check_forward_backward_consistency,
    estimate_motion_field, generate_smart_mask, interpolate_frame, AdjustmentParameters,
    ChromaKeyParams, HdrToneMapParams, HdrTransferFunction, LevelsParameters, Lut3D, LutPreset,
    MotionTracker, OpticalFlowConfig, RgbCurves, SeedPoint, SmartMaskParams, SplinePoint,
    ToneMappingAlgorithm,
};
use iris_export::{
    validate_exported_media, BatchPhotoProcessor, BatchPhotoTask, ExportPreset, StreamCopyOptimizer,
};
use iris_images::{
    apply_exif_orientation, apply_lens_correction, apply_perspective_warp, crop_image,
    decode_image, encode_jpeg, encode_png, encode_webp, flip_horizontal, flip_vertical,
    resize_bilinear, rotate_90, straighten_image, RgbaImage,
};
use iris_media::{HardwareBackendKind, HardwareDeviceContext, VideoFrame};
use iris_projects::{CrashJournal, Project, UndoRedoManager};
use iris_render::{
    render_motion_title, render_multicam_grid, render_rgb_parade, render_vectorscope,
    render_waveform, GpuComputePipeline, MockSourceResolver, MultiCamAngleView, PreviewEngine,
    TitleTemplate,
};
use iris_timeline::{
    Clip, RippleDeleteCommand, SplitClipCommand, SubtitleItem, SubtitleTrack, Track, TrackKind,
    Transition, TransitionKind,
};
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
    for chunk in coord_slice.as_chunks::<2>().0 {
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

#[no_mangle]
pub extern "C" fn iris_audio_process_eq(
    samples: *mut f32,
    count: usize,
    sample_rate: f32,
    low_gain_db: f32,
    mid_gain_db: f32,
    high_gain_db: f32,
) -> c_int {
    if samples.is_null() || count == 0 {
        return -1;
    }
    let slice = unsafe { slice::from_raw_parts_mut(samples, count) };
    let mut eq = ThreeBandEq::new(sample_rate);
    eq.low = EqualizerBand {
        frequency_hz: 120.0,
        gain_db: low_gain_db,
        q_factor: 0.707,
    };
    eq.mid = EqualizerBand {
        frequency_hz: 1000.0,
        gain_db: mid_gain_db,
        q_factor: 1.0,
    };
    eq.high = EqualizerBand {
        frequency_hz: 8000.0,
        gain_db: high_gain_db,
        q_factor: 0.707,
    };
    eq.process_stereo(slice);
    0
}

#[no_mangle]
pub extern "C" fn iris_audio_process_compressor(
    samples: *mut f32,
    count: usize,
    sample_rate: f32,
    threshold_dbfs: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
) -> c_int {
    if samples.is_null() || count == 0 {
        return -1;
    }
    let slice = unsafe { slice::from_raw_parts_mut(samples, count) };
    let comp = AudioCompressor {
        threshold_dbfs,
        ratio,
        attack_ms,
        release_ms,
        makeup_gain_db,
    };
    comp.process_stereo(slice, sample_rate);
    0
}

#[no_mangle]
pub extern "C" fn iris_timeline_add_transition(
    engine: *mut IrisEngine,
    id: *const c_char,
    kind_code: c_int,
    start_num: i64,
    start_den: u64,
    dur_num: i64,
    dur_den: u64,
    track_id: *const c_char,
    clip_a: *const c_char,
    clip_b: *const c_char,
) -> c_int {
    if engine.is_null() {
        return -1;
    }
    let e = unsafe { &mut *engine };
    let id_str = if id.is_null() {
        "trans_default".to_string()
    } else {
        unsafe { CStr::from_ptr(id).to_string_lossy().into_owned() }
    };
    let trk_str = if track_id.is_null() {
        "trk_v1".to_string()
    } else {
        unsafe { CStr::from_ptr(track_id).to_string_lossy().into_owned() }
    };
    let a_str = if clip_a.is_null() {
        "clip_1".to_string()
    } else {
        unsafe { CStr::from_ptr(clip_a).to_string_lossy().into_owned() }
    };
    let b_str = if clip_b.is_null() {
        "clip_2".to_string()
    } else {
        unsafe { CStr::from_ptr(clip_b).to_string_lossy().into_owned() }
    };

    let kind = match kind_code {
        0 => TransitionKind::CrossDissolve,
        1 => TransitionKind::WipeLeft,
        2 => TransitionKind::WipeRight,
        3 => TransitionKind::WipeUp,
        4 => TransitionKind::WipeDown,
        5 => TransitionKind::FadeToBlack,
        6 => TransitionKind::Slide,
        7 => TransitionKind::PushLeft,
        8 => TransitionKind::PushRight,
        9 => TransitionKind::IrisRound,
        10 => TransitionKind::ZoomIn,
        11 => TransitionKind::ZoomOut,
        12 => TransitionKind::DipToColor {
            r: 255,
            g: 255,
            b: 255,
        },
        _ => TransitionKind::CrossDissolve,
    };

    let trans = Transition::new(
        id_str,
        kind,
        RationalTime::new(start_num, start_den),
        RationalTime::new(dur_num, dur_den),
        trk_str,
        a_str,
        b_str,
    );

    e.project.timeline.transitions.retain(|t| t.id != trans.id);
    e.project.timeline.transitions.push(trans);
    0
}

#[no_mangle]
pub extern "C" fn iris_timeline_import_subtitles_srt(
    engine: *mut IrisEngine,
    srt_content: *const c_char,
) -> c_int {
    if engine.is_null() || srt_content.is_null() {
        return -1;
    }
    let e = unsafe { &mut *engine };
    let srt_str = unsafe { CStr::from_ptr(srt_content).to_string_lossy() };
    match SubtitleTrack::parse_srt(&srt_str) {
        Ok(track) => {
            e.project.timeline.subtitle_tracks.clear();
            e.project.timeline.subtitle_tracks.push(track);
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn iris_timeline_export_subtitles_srt(engine: *mut IrisEngine) -> *mut c_char {
    if engine.is_null() {
        return std::ptr::null_mut();
    }
    let e = unsafe { &mut *engine };
    if let Some(track) = e.project.timeline.subtitle_tracks.first() {
        let srt = track.to_srt();
        CString::new(srt)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut())
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn iris_timeline_add_subtitle(
    engine: *mut IrisEngine,
    text: *const c_char,
    start_ms: i64,
    dur_ms: i64,
) -> c_int {
    if engine.is_null() || text.is_null() {
        return -1;
    }
    let e = unsafe { &mut *engine };
    let text_str = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
    let start_time = RationalTime::new(start_ms, 1000);
    let end_time = RationalTime::new(start_ms + dur_ms, 1000);

    if e.project.timeline.subtitle_tracks.is_empty() {
        e.project
            .timeline
            .subtitle_tracks
            .push(SubtitleTrack::new("st_1", "Captions"));
    }

    let track = &mut e.project.timeline.subtitle_tracks[0];
    let id = format!("sub_{}", track.items.len() + 1);
    track.add_item(SubtitleItem::new(id, start_time, end_time, text_str));
    0
}

#[no_mangle]
pub extern "C" fn iris_export_analyze_stream_copy(
    engine: *mut IrisEngine,
    out_can_full_copy: *mut c_int,
    out_copy_percentage: *mut f32,
    out_speedup_factor: *mut f32,
) -> c_int {
    if engine.is_null() {
        return -1;
    }
    let e = unsafe { &mut *engine };
    let config = ExportPreset::Web1080p.to_config("test.mp4");
    let plan = StreamCopyOptimizer::analyze_timeline(&e.project.timeline, &config);

    if !out_can_full_copy.is_null() {
        unsafe { *out_can_full_copy = if plan.can_full_stream_copy { 1 } else { 0 } };
    }
    if !out_copy_percentage.is_null() {
        unsafe { *out_copy_percentage = plan.stream_copy_percentage };
    }
    if !out_speedup_factor.is_null() {
        unsafe { *out_speedup_factor = plan.estimated_speedup };
    }
    0
}

#[no_mangle]
pub extern "C" fn iris_lut_apply_preset(
    pixels: *mut u8,
    len: usize,
    preset_idx: c_int,
    intensity: f32,
) -> c_int {
    if pixels.is_null() || len == 0 || !len.is_multiple_of(4) {
        return -1;
    }
    let preset = match preset_idx {
        0 => LutPreset::TealAndOrange,
        1 => LutPreset::WarmVintage,
        2 => LutPreset::BleachBypass,
        3 => LutPreset::FadedFilm,
        _ => LutPreset::MonochromeContrast,
    };
    let lut = Lut3D::preset(preset);
    let px = unsafe { slice::from_raw_parts_mut(pixels, len) };
    lut.apply_rgba(px, intensity);
    0
}

#[no_mangle]
pub extern "C" fn iris_lut_apply_cube(
    pixels: *mut u8,
    len: usize,
    cube_content: *const c_char,
    intensity: f32,
) -> c_int {
    if pixels.is_null() || len == 0 || !len.is_multiple_of(4) || cube_content.is_null() {
        return -1;
    }
    let c_str = unsafe { CStr::from_ptr(cube_content) };
    let content = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let lut = match Lut3D::parse_cube(content) {
        Ok(l) => l,
        Err(_) => return -3,
    };
    let px = unsafe { slice::from_raw_parts_mut(pixels, len) };
    lut.apply_rgba(px, intensity);
    0
}

#[no_mangle]
pub extern "C" fn iris_curves_apply_rgb(
    pixels: *mut u8,
    len: usize,
    master_pts: *const f32,
    master_count: usize,
    r_pts: *const f32,
    r_count: usize,
    g_pts: *const f32,
    g_count: usize,
    b_pts: *const f32,
    b_count: usize,
) -> c_int {
    if pixels.is_null() || len == 0 || !len.is_multiple_of(4) {
        return -1;
    }

    let parse_pts = |ptr: *const f32, count: usize| -> Vec<SplinePoint> {
        if ptr.is_null() || count == 0 {
            return vec![
                SplinePoint { x: 0.0, y: 0.0 },
                SplinePoint { x: 1.0, y: 1.0 },
            ];
        }
        let s = unsafe { slice::from_raw_parts(ptr, count * 2) };
        s.as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| SplinePoint {
                x: chunk[0],
                y: chunk[1],
            })
            .collect()
    };

    let m_points = parse_pts(master_pts, master_count);
    let r_points = parse_pts(r_pts, r_count);
    let g_points = parse_pts(g_pts, g_count);
    let b_points = parse_pts(b_pts, b_count);

    let curves = RgbCurves::new(m_points, r_points, g_points, b_points);
    let px = unsafe { slice::from_raw_parts_mut(pixels, len) };
    curves.apply_rgba(px);
    0
}

#[no_mangle]
pub extern "C" fn iris_audio_time_stretch(
    samples: *const f32,
    sample_count: usize,
    channels: c_int,
    sample_rate: c_int,
    speed_ratio: f32,
    out_ptr: *mut *mut f32,
    out_len: *mut usize,
) -> c_int {
    if samples.is_null() || sample_count == 0 || out_ptr.is_null() || out_len.is_null() {
        return -1;
    }
    let ch = channels.max(1) as usize;
    let sr = sample_rate.max(8000) as u32;
    let wsola = WsolaTimeStretch::new(sr, ch);
    let in_slice = unsafe { slice::from_raw_parts(samples, sample_count) };
    let stretched = wsola.process(in_slice, speed_ratio);

    let len = stretched.len();
    let mut boxed = stretched.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);

    unsafe {
        *out_ptr = ptr;
        *out_len = len;
    }
    0
}

#[no_mangle]
pub extern "C" fn iris_audio_buffer_free(ptr: *mut f32, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len));
        }
    }
}

#[no_mangle]
pub extern "C" fn iris_audio_duck_music(
    dialogue_samples: *const f32,
    music_samples: *mut f32,
    sample_count: usize,
    channels: c_int,
    sample_rate: c_int,
    threshold_db: f32,
    attenuation_db: f32,
) -> c_int {
    if dialogue_samples.is_null() || music_samples.is_null() || sample_count == 0 {
        return -1;
    }
    let ch = channels.max(1) as usize;
    let sr = sample_rate.max(8000) as u32;
    let mut ducker = AudioDucker::new(
        sr,
        DuckingParameters {
            threshold_db,
            duck_attenuation_db: attenuation_db,
            attack_ms: 20.0,
            hold_ms: 300.0,
            release_ms: 450.0,
        },
    );
    let diag = unsafe { slice::from_raw_parts(dialogue_samples, sample_count) };
    let music = unsafe { slice::from_raw_parts_mut(music_samples, sample_count) };
    ducker.process_interleaved(diag, music, ch);
    0
}

#[no_mangle]
pub extern "C" fn iris_scopes_render(
    src_pixels: *const u8,
    src_w: u32,
    src_h: u32,
    scope_kind: c_int, // 0 = Waveform, 1 = Vectorscope, 2 = RGB Parade
    out_w: u32,
    out_h: u32,
    out_pixels: *mut u8,
) -> c_int {
    if src_pixels.is_null()
        || out_pixels.is_null()
        || src_w == 0
        || src_h == 0
        || out_w == 0
        || out_h == 0
    {
        return -1;
    }

    let src_slice = unsafe { slice::from_raw_parts(src_pixels, (src_w * src_h * 4) as usize) };
    let src_frame = VideoFrame::from_raw(
        Dimensions::new(src_w, src_h),
        PixelFormat::Rgba8,
        RationalTime::new(0, 30),
        RationalTime::new(1, 30),
        src_slice.to_vec(),
    );

    let scope_frame = match scope_kind {
        0 => render_waveform(&src_frame, out_w, out_h),
        1 => render_vectorscope(&src_frame, out_w.min(out_h)),
        2 => render_rgb_parade(&src_frame, out_w, out_h),
        _ => return -2,
    };

    let out_slice = unsafe {
        slice::from_raw_parts_mut(
            out_pixels,
            (scope_frame.dimensions.width * scope_frame.dimensions.height * 4) as usize,
        )
    };
    out_slice.copy_from_slice(&scope_frame.data);
    0
}

// =========================================================================
// Phase 4: Chroma Keying, Motion Tracking, Hardware Acceleration, Motion Titles
// =========================================================================

#[no_mangle]
pub extern "C" fn iris_chroma_key_apply(
    pixels: *mut u8,
    len: usize,
    key_r: f32,
    key_g: f32,
    key_b: f32,
    similarity: f32,
    smoothness: f32,
    spill_suppression: f32,
) -> c_int {
    if pixels.is_null() || len == 0 || !len.is_multiple_of(4) {
        return -1;
    }
    let slice = unsafe { slice::from_raw_parts_mut(pixels, len) };
    let params = ChromaKeyParams {
        key_color: [key_r, key_g, key_b],
        similarity,
        smoothness,
        spill_suppression,
    };
    apply_chroma_key(slice, &params);
    0
}

#[repr(C)]
pub struct CMotionTrackResult {
    pub center_x: f32,
    pub center_y: f32,
    pub confidence: f32,
}

#[no_mangle]
pub extern "C" fn iris_motion_track_frame(
    frame_a: *const u8,
    frame_b: *const u8,
    width: usize,
    height: usize,
    center_x: f32,
    center_y: f32,
    patch_size: usize,
    search_radius: usize,
    smoothing: f32,
    out_result: *mut CMotionTrackResult,
) -> c_int {
    if frame_a.is_null()
        || frame_b.is_null()
        || out_result.is_null()
        || width == 0
        || height == 0
        || patch_size == 0
    {
        return -1;
    }
    let len = width * height * 4;
    let slice_a = unsafe { slice::from_raw_parts(frame_a, len) };
    let slice_b = unsafe { slice::from_raw_parts(frame_b, len) };

    let tracker = MotionTracker::new(search_radius, smoothing);
    let pt = tracker.track_between_frames(
        slice_a,
        slice_b,
        width,
        height,
        (center_x, center_y),
        patch_size,
    );

    unsafe {
        (*out_result).center_x = pt.center_x;
        (*out_result).center_y = pt.center_y;
        (*out_result).confidence = pt.confidence;
    }
    0
}

#[repr(C)]
pub struct CHardwareDeviceInfo {
    pub backend_code: c_int,
    pub supports_zero_copy: c_int,
    pub max_width: u32,
    pub max_height: u32,
    pub max_streams: usize,
    pub is_available: c_int,
}

#[no_mangle]
pub extern "C" fn iris_hardware_detect(
    out_info: *mut CHardwareDeviceInfo,
    out_name_buf: *mut c_char,
    name_buf_len: usize,
) -> c_int {
    if out_info.is_null() {
        return -1;
    }
    let ctx = HardwareDeviceContext::detect();
    let backend_code = match ctx.backend {
        HardwareBackendKind::D3D11VA => 0,
        HardwareBackendKind::DXVA2 => 1,
        HardwareBackendKind::VideoToolbox => 2,
        HardwareBackendKind::VaApi => 3,
        HardwareBackendKind::MediaCodec => 4,
        HardwareBackendKind::Software => 5,
    };

    unsafe {
        (*out_info).backend_code = backend_code;
        (*out_info).supports_zero_copy = if ctx.supports_zero_copy { 1 } else { 0 };
        (*out_info).max_width = ctx.max_width;
        (*out_info).max_height = ctx.max_height;
        (*out_info).max_streams = ctx.max_concurrent_streams;
        (*out_info).is_available = if ctx.is_available { 1 } else { 0 };

        if !out_name_buf.is_null() && name_buf_len > 0 {
            let name_bytes = ctx.device_name.as_bytes();
            let copy_len = name_bytes.len().min(name_buf_len - 1);
            std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), out_name_buf as *mut u8, copy_len);
            *out_name_buf.add(copy_len) = 0;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn iris_render_motion_title(
    pixels: *mut u8,
    width: u32,
    height: u32,
    title: *const c_char,
    subtitle: *const c_char,
    template_idx: c_int,
    progress: f32,
) -> c_int {
    if pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let title_str = if title.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(title).to_str().unwrap_or("") }
    };
    let sub_str = if subtitle.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(subtitle).to_str().unwrap_or("") }
    };

    let template = match template_idx {
        0 => TitleTemplate::MinimalClean,
        1 => TitleTemplate::LowerThirdNews,
        2 => TitleTemplate::CinematicCenter,
        3 => TitleTemplate::CalloutBox,
        _ => TitleTemplate::MinimalClean,
    };

    let len = (width * height * 4) as usize;
    let slice = unsafe { slice::from_raw_parts_mut(pixels, len) };
    let mut frame = VideoFrame::from_raw(
        Dimensions::new(width, height),
        PixelFormat::Rgba8,
        RationalTime::ZERO,
        RationalTime::new(1, 30),
        slice.to_vec(),
    );

    render_motion_title(&mut frame, title_str, sub_str, template, progress);

    slice.copy_from_slice(&frame.data);
    0
}

// =========================================================================
// Phase 5: Multi-Camera Switching, Spatial Audio, HDR & GPU Compute Pipeline
// =========================================================================

#[no_mangle]
pub extern "C" fn iris_spatial_audio_pan_binaural(
    mono_samples: *const f32,
    stereo_out: *mut f32,
    sample_count: usize,
    sample_rate: c_int,
    azimuth_deg: f32,
    elevation_deg: f32,
    distance_meters: f32,
) -> c_int {
    if mono_samples.is_null() || stereo_out.is_null() || sample_count == 0 {
        return -1;
    }
    let sr = sample_rate.max(8000) as u32;
    let mut panner = BinauralPanner::new(sr);
    let mono_slice = unsafe { slice::from_raw_parts(mono_samples, sample_count) };
    let stereo_slice = unsafe { slice::from_raw_parts_mut(stereo_out, sample_count * 2) };

    panner.process_mono_to_binaural(
        mono_slice,
        stereo_slice,
        azimuth_deg,
        elevation_deg,
        distance_meters,
    );
    0
}

#[no_mangle]
pub extern "C" fn iris_hdr_tonemap_frame(
    pixels: *mut u8,
    len: usize,
    transfer_fn: c_int, // 0 = PQ, 1 = HLG, 2 = Linear
    algorithm: c_int,   // 0 = ACES Filmic, 1 = Reinhard, 2 = Knee
    exposure_stops: f32,
    target_nits: f32,
    white_point: f32,
) -> c_int {
    if pixels.is_null() || len == 0 || !len.is_multiple_of(4) {
        return -1;
    }
    let slice = unsafe { slice::from_raw_parts_mut(pixels, len) };

    let tf = match transfer_fn {
        0 => HdrTransferFunction::Pq,
        1 => HdrTransferFunction::Hlg,
        _ => HdrTransferFunction::Linear,
    };
    let algo = match algorithm {
        0 => ToneMappingAlgorithm::AcesFilmic,
        1 => ToneMappingAlgorithm::ReinhardExtended,
        _ => ToneMappingAlgorithm::PhotographicKnee,
    };

    let params = HdrToneMapParams {
        transfer_function: tf,
        algorithm: algo,
        exposure_stops,
        target_nits,
        white_point,
    };

    apply_hdr_tonemap(slice, &params);
    0
}

#[no_mangle]
pub extern "C" fn iris_gpu_compute_dispatch_tonemap(
    in_pixels: *const u8,
    out_pixels: *mut u8,
    len: usize,
    exposure_stops: f32,
) -> c_int {
    if in_pixels.is_null() || out_pixels.is_null() || len == 0 || !len.is_multiple_of(4) {
        return -1;
    }
    let in_slice = unsafe { slice::from_raw_parts(in_pixels, len) };
    let out_slice = unsafe { slice::from_raw_parts_mut(out_pixels, len) };

    let pipeline = GpuComputePipeline::new();
    pipeline.dispatch_tonemap(in_slice, out_slice, exposure_stops);
    0
}

#[no_mangle]
pub extern "C" fn iris_multicam_render_quad(
    in_pixels_1: *const u8,
    in_pixels_2: *const u8,
    in_pixels_3: *const u8,
    in_pixels_4: *const u8,
    in_w: u32,
    in_h: u32,
    active_angle_idx: c_int,
    out_pixels: *mut u8,
    out_w: u32,
    out_h: u32,
) -> c_int {
    if out_pixels.is_null() || out_w == 0 || out_h == 0 || in_w == 0 || in_h == 0 {
        return -1;
    }

    let in_len = (in_w * in_h * 4) as usize;
    let make_frame = |ptr: *const u8| -> VideoFrame {
        if ptr.is_null() {
            VideoFrame::new(
                Dimensions::new(in_w, in_h),
                PixelFormat::Rgba8,
                RationalTime::ZERO,
                RationalTime::new(1, 30),
            )
        } else {
            let slice = unsafe { slice::from_raw_parts(ptr, in_len) };
            VideoFrame::from_raw(
                Dimensions::new(in_w, in_h),
                PixelFormat::Rgba8,
                RationalTime::ZERO,
                RationalTime::new(1, 30),
                slice.to_vec(),
            )
        }
    };

    let f1 = make_frame(in_pixels_1);
    let f2 = make_frame(in_pixels_2);
    let f3 = make_frame(in_pixels_3);
    let f4 = make_frame(in_pixels_4);

    let angle_views = [
        MultiCamAngleView {
            frame: &f1,
            name: "WIDE",
            is_active: active_angle_idx == 0,
        },
        MultiCamAngleView {
            frame: &f2,
            name: "CLOSE-UP",
            is_active: active_angle_idx == 1,
        },
        MultiCamAngleView {
            frame: &f3,
            name: "DRONE",
            is_active: active_angle_idx == 2,
        },
        MultiCamAngleView {
            frame: &f4,
            name: "STAGE",
            is_active: active_angle_idx == 3,
        },
    ];

    let quad_frame = render_multicam_grid(&angle_views, out_w, out_h);
    let out_len = (out_w * out_h * 4) as usize;
    let out_slice = unsafe { slice::from_raw_parts_mut(out_pixels, out_len) };
    out_slice.copy_from_slice(&quad_frame.data);
    0
}

#[no_mangle]
pub extern "C" fn iris_optical_flow_interpolate(
    frame_0: *const u8,
    frame_1: *const u8,
    out_pixels: *mut u8,
    width: u32,
    height: u32,
    t: f32,
) -> c_int {
    if frame_0.is_null() || frame_1.is_null() || out_pixels.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let w = width as usize;
    let h = height as usize;
    let len = w * h * 4;

    let f0 = unsafe { slice::from_raw_parts(frame_0, len) };
    let f1 = unsafe { slice::from_raw_parts(frame_1, len) };
    let out = unsafe { slice::from_raw_parts_mut(out_pixels, len) };

    let config = OpticalFlowConfig::default();
    let mut fwd = estimate_motion_field(f0, f1, w, h, &config);
    let bwd = estimate_motion_field(f1, f0, w, h, &config);
    check_forward_backward_consistency(&mut fwd, &bwd, config.occlusion_threshold);

    interpolate_frame(f0, f1, out, w, h, &fwd, &bwd, t);
    0
}

#[no_mangle]
pub extern "C" fn iris_smart_mask_generate(
    src_rgba: *const u8,
    out_mask: *mut u8,
    width: u32,
    height: u32,
    seed_x: u32,
    seed_y: u32,
    contrast_threshold: f32,
    color_tolerance: f32,
    feather_radius: u32,
    invert: c_int,
) -> c_int {
    if src_rgba.is_null() || out_mask.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let w = width as usize;
    let h = height as usize;
    let in_len = w * h * 4;
    let out_len = w * h;

    let src = unsafe { slice::from_raw_parts(src_rgba, in_len) };
    let out = unsafe { slice::from_raw_parts_mut(out_mask, out_len) };

    let seeds = vec![SeedPoint {
        x: seed_x.min(width - 1),
        y: seed_y.min(height - 1),
        is_foreground: true,
    }];

    let params = SmartMaskParams {
        contrast_threshold: if contrast_threshold > 0.0 {
            contrast_threshold
        } else {
            0.15
        },
        color_tolerance: if color_tolerance > 0.0 {
            color_tolerance
        } else {
            0.25
        },
        feather_radius: feather_radius as usize,
        invert: invert != 0,
        iterations: 3,
    };

    let mask = generate_smart_mask(src, w, h, &seeds, None, &params);
    out.copy_from_slice(&mask);
    0
}

#[no_mangle]
pub extern "C" fn iris_smart_mask_apply(
    src_rgba: *const u8,
    mask: *const u8,
    out_rgba: *mut u8,
    width: u32,
    height: u32,
) -> c_int {
    if src_rgba.is_null() || mask.is_null() || out_rgba.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let w = width as usize;
    let h = height as usize;
    let rgba_len = w * h * 4;
    let mask_len = w * h;

    let src = unsafe { slice::from_raw_parts(src_rgba, rgba_len) };
    let m = unsafe { slice::from_raw_parts(mask, mask_len) };
    let out = unsafe { slice::from_raw_parts_mut(out_rgba, rgba_len) };

    apply_smart_mask(src, m, out, w, h);
    0
}
