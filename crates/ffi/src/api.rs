#![allow(clippy::not_unsafe_ptr_arg_deref)]

use iris_audio::{generate_waveform_peaks, WaveformPeak};
use iris_cache::{CacheManager, MemoryBudgetTier};
use iris_core::{Dimensions, RationalTime, Rect};
use iris_effects::{apply_adjustments, AdjustmentParameters};
use iris_images::{crop_image, RgbaImage};
use iris_projects::{CrashJournal, Project, UndoRedoManager};
use iris_render::{MockSourceResolver, PreviewEngine};
use iris_timeline::{Clip, RippleDeleteCommand, SplitClipCommand, Track, TrackKind};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
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
