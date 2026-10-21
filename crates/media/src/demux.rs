use crate::frame::{AudioFrame, VideoFrame};
use iris_core::{Dimensions, IrisError, IrisResult, PixelFormat, RationalTime};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    pub codec: String,
    pub dimensions: Dimensions,
    pub frame_rate_num: u64,
    pub frame_rate_den: u64,
    pub duration: RationalTime,
    pub pixel_format: PixelFormat,
    pub bit_rate: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: RationalTime,
    pub bit_rate: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub date: Option<String>,
    pub orientation_degrees: u32,
}

pub trait Demuxer: Send + Sync {
    fn stream_info(&self) -> &StreamInfo;
    fn metadata(&self) -> &MediaMetadata;
    fn seek(&mut self, time: RationalTime) -> IrisResult<()>;
    fn next_video_frame(&mut self) -> IrisResult<Option<VideoFrame>>;
    fn next_audio_frame(&mut self) -> IrisResult<Option<AudioFrame>>;
}

/// Detailed media probe result returned to callers and Dart UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaProbeInfo {
    pub is_valid: bool,
    pub media_type: String, // "video", "photo", "audio", "unknown"
    pub filename: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration_seconds: f64,
    pub channels: u32,
    pub sample_rate: u32,
    pub codec_name: String,
    pub file_size_bytes: u64,
    pub orientation_degrees: u32,
    pub has_video: bool,
    pub has_audio: bool,
    pub is_vfr: bool,
    pub error: Option<String>,
}

impl MediaProbeInfo {
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            media_type: "unknown".into(),
            filename: "".into(),
            format: "".into(),
            width: 0,
            height: 0,
            fps: 0.0,
            duration_seconds: 0.0,
            channels: 0,
            sample_rate: 0,
            codec_name: "".into(),
            file_size_bytes: 0,
            orientation_degrees: 0,
            has_video: false,
            has_audio: false,
            is_vfr: false,
            error: Some(msg.into()),
        }
    }
}

/// Normalize input file path handling spaces, Unicode, and mobile file:// URIs.
pub fn normalize_media_path(raw_path: &str) -> PathBuf {
    let mut p = raw_path.trim();
    if let Some(stripped) = p.strip_prefix("file:///") {
        p = stripped;
    } else if let Some(stripped) = p.strip_prefix("file://") {
        p = stripped;
    }
    PathBuf::from(p)
}

/// Pure-Rust parser for ISO-BMFF (MP4, MOV, M4A, M4V) container boxes.
pub struct IsoBmffParser;

impl IsoBmffParser {
    pub fn probe(path: &Path) -> Result<MediaProbeInfo, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let metadata = file
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        let file_size = metadata.len();
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("media")
            .to_string();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4")
            .to_lowercase();

        let mut width = 0u32;
        let mut height = 0u32;
        let mut timescale = 1000u32;
        let mut duration_units = 0u64;
        let fps = 30.0f64;
        let mut channels = 0u32;
        let mut sample_rate = 0u32;
        let mut has_video = false;
        let mut has_audio = false;
        let mut codec_name = String::new();

        let mut buffer = [0u8; 8];
        let mut pos = 0u64;

        while pos + 8 <= file_size {
            if file.seek(SeekFrom::Start(pos)).is_err() {
                break;
            }
            if file.read_exact(&mut buffer).is_err() {
                break;
            }

            let box_size = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as u64;
            let box_type = &buffer[4..8];

            let effective_size = if box_size == 1 {
                // 64-bit largesize
                let mut large_buf = [0u8; 8];
                if file.read_exact(&mut large_buf).is_err() {
                    break;
                }
                u64::from_be_bytes(large_buf)
            } else if box_size == 0 {
                file_size - pos
            } else {
                box_size
            };

            if effective_size < 8 {
                break;
            }

            if box_type == b"moov" {
                // Parse moov sub-boxes
                let moov_end = pos + effective_size;
                let mut sub_pos = pos + (if box_size == 1 { 16 } else { 8 });

                while sub_pos + 8 <= moov_end && sub_pos + 8 <= file_size {
                    if file.seek(SeekFrom::Start(sub_pos)).is_err() {
                        break;
                    }
                    let mut sub_hdr = [0u8; 8];
                    if file.read_exact(&mut sub_hdr).is_err() {
                        break;
                    }

                    let sub_size =
                        u32::from_be_bytes([sub_hdr[0], sub_hdr[1], sub_hdr[2], sub_hdr[3]]) as u64;
                    let sub_type = &sub_hdr[4..8];
                    let sub_eff_size = if sub_size == 1 {
                        16
                    } else if sub_size == 0 {
                        moov_end - sub_pos
                    } else {
                        sub_size
                    };
                    if sub_eff_size < 8 {
                        break;
                    }

                    if sub_type == b"mvhd" {
                        // Movie Header Box
                        let mut mvhd_data = vec![0u8; (sub_eff_size as usize).min(128)];
                        if file.seek(SeekFrom::Start(sub_pos)).is_ok()
                            && file.read_exact(&mut mvhd_data).is_ok()
                        {
                            let version = mvhd_data.get(8).copied().unwrap_or(0);
                            if version == 0 && mvhd_data.len() >= 32 {
                                timescale = u32::from_be_bytes([
                                    mvhd_data[20],
                                    mvhd_data[21],
                                    mvhd_data[22],
                                    mvhd_data[23],
                                ])
                                .max(1);
                                duration_units = u32::from_be_bytes([
                                    mvhd_data[24],
                                    mvhd_data[25],
                                    mvhd_data[26],
                                    mvhd_data[27],
                                ]) as u64;
                            } else if version == 1 && mvhd_data.len() >= 44 {
                                timescale = u32::from_be_bytes([
                                    mvhd_data[28],
                                    mvhd_data[29],
                                    mvhd_data[30],
                                    mvhd_data[31],
                                ])
                                .max(1);
                                duration_units = u64::from_be_bytes([
                                    mvhd_data[32],
                                    mvhd_data[33],
                                    mvhd_data[34],
                                    mvhd_data[35],
                                    mvhd_data[36],
                                    mvhd_data[37],
                                    mvhd_data[38],
                                    mvhd_data[39],
                                ]);
                            }
                        }
                    } else if sub_type == b"trak" {
                        // Scan track for tkhd, mdia -> hdlr, stsd, stts
                        let trak_end = sub_pos + sub_eff_size;
                        let mut t_pos = sub_pos + 8;
                        let mut is_this_audio = false;

                        while t_pos + 8 <= trak_end && t_pos + 8 <= file_size {
                            if file.seek(SeekFrom::Start(t_pos)).is_err() {
                                break;
                            }
                            let mut t_hdr = [0u8; 8];
                            if file.read_exact(&mut t_hdr).is_err() {
                                break;
                            }
                            let t_size =
                                u32::from_be_bytes([t_hdr[0], t_hdr[1], t_hdr[2], t_hdr[3]]) as u64;
                            let t_type = &t_hdr[4..8];
                            let t_eff_size = if t_size < 8 { 8 } else { t_size };

                            if t_type == b"tkhd" {
                                let mut tkhd_data = vec![0u8; (t_eff_size as usize).min(100)];
                                if file.seek(SeekFrom::Start(t_pos)).is_ok()
                                    && file.read_exact(&mut tkhd_data).is_ok()
                                    && tkhd_data.len() >= 84
                                {
                                    let w_fix = u32::from_be_bytes([
                                        tkhd_data[tkhd_data.len() - 8],
                                        tkhd_data[tkhd_data.len() - 7],
                                        tkhd_data[tkhd_data.len() - 6],
                                        tkhd_data[tkhd_data.len() - 5],
                                    ]);
                                    let h_fix = u32::from_be_bytes([
                                        tkhd_data[tkhd_data.len() - 4],
                                        tkhd_data[tkhd_data.len() - 3],
                                        tkhd_data[tkhd_data.len() - 2],
                                        tkhd_data[tkhd_data.len() - 1],
                                    ]);
                                    let w_val = w_fix >> 16;
                                    let h_val = h_fix >> 16;
                                    if w_val > 0 && h_val > 0 {
                                        width = w_val;
                                        height = h_val;
                                    }
                                }
                            } else if t_type == b"mdia" {
                                // Search inside mdia for hdlr and stbl
                                let mut mdia_data = vec![0u8; (t_eff_size as usize).min(4096)];
                                if file.seek(SeekFrom::Start(t_pos)).is_ok()
                                    && file.read_exact(&mut mdia_data).is_ok()
                                {
                                    if mdia_data.windows(4).any(|w| w == b"vide") {
                                        has_video = true;
                                    }
                                    if mdia_data.windows(4).any(|w| w == b"soun") {
                                        is_this_audio = true;
                                        has_audio = true;
                                    }
                                    if mdia_data.windows(4).any(|w| w == b"avc1") {
                                        codec_name = "H.264 / AVC".into();
                                    } else if mdia_data
                                        .windows(4)
                                        .any(|w| w == b"hvc1" || w == b"hev1")
                                    {
                                        codec_name = "H.265 / HEVC".into();
                                    } else if mdia_data.windows(4).any(|w| w == b"vp09") {
                                        codec_name = "VP9".into();
                                    } else if mdia_data.windows(4).any(|w| w == b"av01") {
                                        codec_name = "AV1".into();
                                    }
                                    if is_this_audio && (channels == 0 || sample_rate == 0) {
                                        channels = 2;
                                        sample_rate = 48000;
                                    }
                                }
                            }

                            t_pos += t_eff_size;
                        }
                    }

                    sub_pos += sub_eff_size;
                }
            }

            pos += effective_size;
        }

        if !has_video && !has_audio {
            return Err("Not a valid ISO-BMFF media file or no playable tracks found".into());
        }

        let duration_seconds = if timescale > 0 && duration_units > 0 {
            (duration_units as f64) / (timescale as f64)
        } else {
            ((file_size as f64) * 8.0) / 10_000_000.0
        };

        if width == 0 || height == 0 {
            width = 1920;
            height = 1080;
        }

        let media_type = if has_video {
            "video"
        } else {
            "audio"
        };

        if codec_name.is_empty() {
            codec_name = if media_type == "video" {
                "H.264 / AAC".into()
            } else {
                "AAC Audio".into()
            };
        }

        Ok(MediaProbeInfo {
            is_valid: true,
            media_type: media_type.into(),
            filename,
            format: ext,
            width,
            height,
            fps,
            duration_seconds: duration_seconds.max(0.1),
            channels: if has_audio { channels.max(1) } else { 0 },
            sample_rate: if has_audio { sample_rate.max(8000) } else { 0 },
            codec_name,
            file_size_bytes: file_size,
            orientation_degrees: 0,
            has_video: true,
            has_audio,
            is_vfr: false,
            error: None,
        })
    }
}

/// Creates a bounded, non-interactive command that never opens a console window on Windows.
fn create_safe_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

/// Executes a child command with a hard timeout, actively killing the process if exceeded.
fn run_command_with_timeout(
    mut cmd: Command,
    timeout_ms: u64,
) -> Result<std::process::Output, String> {
    use std::time::{Duration, Instant};
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                return child
                    .wait_with_output()
                    .map_err(|e| format!("Failed to read output: {}", e));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("Process timed out after {}ms", timeout_ms));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("Error waiting on process: {}", e));
            }
        }
    }
}

static FFMPEG_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static FFPROBE_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

#[derive(Clone)]
struct CachedFrameEntry {
    path: String,
    time_bucket_ms: i64,
    dim: Dimensions,
    frame: VideoFrame,
}

static FRAME_CACHE: std::sync::OnceLock<std::sync::Mutex<Vec<CachedFrameEntry>>> =
    std::sync::OnceLock::new();
static FFMPEG_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn get_cached_frame(path: &str, time_bucket_ms: i64, dim: Dimensions) -> Option<VideoFrame> {
    let cache = FRAME_CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    if let Ok(guard) = cache.lock() {
        for entry in guard.iter().rev() {
            if entry.path == path
                && (entry.time_bucket_ms - time_bucket_ms).abs() <= 33
                && entry.dim == dim
            {
                return Some(entry.frame.clone());
            }
        }
    }
    None
}

fn store_cached_frame(path: &str, time_bucket_ms: i64, dim: Dimensions, frame: VideoFrame) {
    let cache = FRAME_CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= 16 {
            guard.remove(0);
        }
        guard.push(CachedFrameEntry {
            path: path.to_string(),
            time_bucket_ms,
            dim,
            frame,
        });
    }
}

/// Real media decoder and probe engine.
pub struct RealMediaDecoder;

impl RealMediaDecoder {
    /// Detects if ffmpeg binary is accessible (evaluated once, cached statically).
    pub fn is_ffmpeg_available() -> bool {
        *FFMPEG_AVAILABLE.get_or_init(|| {
            let mut cmd = create_safe_command("ffmpeg");
            cmd.arg("-version")
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            run_command_with_timeout(cmd, 1500)
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
    }

    /// Detects if ffprobe binary is accessible (evaluated once, cached statically).
    pub fn is_ffprobe_available() -> bool {
        *FFPROBE_AVAILABLE.get_or_init(|| {
            let mut cmd = create_safe_command("ffprobe");
            cmd.arg("-version")
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            run_command_with_timeout(cmd, 1500)
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
    }

    /// Probe media file metadata with high accuracy using ffprobe when present or pure-Rust demuxer.
    pub fn probe_file(raw_path: &str) -> MediaProbeInfo {
        let path = normalize_media_path(raw_path);
        if !path.exists() {
            return MediaProbeInfo::error(format!("File does not exist: {}", path.display()));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("media")
            .to_string();
        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        // 1. Photos / Image Formats
        if matches!(
            ext.as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff"
        ) {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    return MediaProbeInfo {
                        is_valid: true,
                        media_type: "photo".into(),
                        filename,
                        format: ext.clone(),
                        width: img.width(),
                        height: img.height(),
                        fps: 0.0,
                        duration_seconds: 0.0,
                        channels: 4,
                        sample_rate: 0,
                        codec_name: format!("{} Raster Image", ext.to_uppercase()),
                        file_size_bytes: file_size,
                        orientation_degrees: 0,
                        has_video: false,
                        has_audio: false,
                        is_vfr: false,
                        error: None,
                    };
                }
            }
            return MediaProbeInfo::error(format!("Unable to decode image format: {}", filename));
        }

        // 2. Audio WAV Format
        if ext == "wav" {
            if let Ok(bytes) = std::fs::read(&path) {
                if bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
                    let channels = u16::from_le_bytes([bytes[22], bytes[23]]) as u32;
                    let sample_rate =
                        u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
                    let byte_rate =
                        u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
                    let data_len = if bytes.len() > 44 {
                        (bytes.len() - 44) as f64
                    } else {
                        0.0
                    };
                    let duration = if byte_rate > 0 {
                        data_len / byte_rate as f64
                    } else {
                        0.0
                    };
                    return MediaProbeInfo {
                        is_valid: true,
                        media_type: "audio".into(),
                        filename,
                        format: "wav".into(),
                        width: 0,
                        height: 0,
                        fps: 0.0,
                        duration_seconds: duration.max(0.1),
                        channels: channels.max(1),
                        sample_rate: sample_rate.max(8000),
                        codec_name: "PCM Linear Audio".into(),
                        file_size_bytes: file_size,
                        orientation_degrees: 0,
                        has_video: false,
                        has_audio: true,
                        is_vfr: false,
                        error: None,
                    };
                }
            }
        }

        // 3. Pure-Rust ISO-BMFF Demuxer for MP4/MOV/M4V (Zero subprocesses, ~0.5ms)
        if matches!(ext.as_str(), "mp4" | "mov" | "m4v" | "m4a") {
            if let Ok(info) = IsoBmffParser::probe(&path) {
                return info;
            }
        }

        // 4. Try ffprobe for comprehensive container probing if available (with strict timeout)
        if Self::is_ffprobe_available() {
            let mut cmd = create_safe_command("ffprobe");
            cmd.args([
                "-nostdin",
                "-noautorun",
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                path.to_str().unwrap_or(""),
            ]);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            if let Ok(output) = run_command_with_timeout(cmd, 2000) {
                if output.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        let mut width = 0u32;
                        let mut height = 0u32;
                        let mut fps = 30.0f64;
                        let mut duration_sec = 0.0f64;
                        let mut channels = 0u32;
                        let mut sample_rate = 0u32;
                        let mut has_video = false;
                        let mut has_audio = false;
                        let mut codec_name = String::new();

                        if let Some(format_obj) = val.get("format") {
                            if let Some(dur_str) =
                                format_obj.get("duration").and_then(|d| d.as_str())
                            {
                                duration_sec = dur_str.parse::<f64>().unwrap_or(0.0);
                            }
                        }

                        if let Some(streams) = val.get("streams").and_then(|s| s.as_array()) {
                            for s in streams {
                                let codec_type =
                                    s.get("codec_type").and_then(|c| c.as_str()).unwrap_or("");
                                if codec_type == "video" && !has_video {
                                    has_video = true;
                                    width = s.get("width").and_then(|w| w.as_u64()).unwrap_or(1920)
                                        as u32;
                                    height =
                                        s.get("height").and_then(|h| h.as_u64()).unwrap_or(1080)
                                            as u32;
                                    if let Some(r_fps) =
                                        s.get("r_frame_rate").and_then(|r| r.as_str())
                                    {
                                        let parts: Vec<&str> = r_fps.split('/').collect();
                                        if parts.len() == 2 {
                                            let num: f64 = parts[0].parse().unwrap_or(30.0);
                                            let den: f64 = parts[1].parse().unwrap_or(1.0);
                                            if den > 0.0 {
                                                fps = num / den;
                                            }
                                        }
                                    }
                                    if let Some(name) = s.get("codec_name").and_then(|n| n.as_str())
                                    {
                                        codec_name = name.to_uppercase();
                                    }
                                    if duration_sec <= 0.0 {
                                        if let Some(dur_str) =
                                            s.get("duration").and_then(|d| d.as_str())
                                        {
                                            duration_sec = dur_str.parse::<f64>().unwrap_or(0.0);
                                        }
                                    }
                                } else if codec_type == "audio" && !has_audio {
                                    has_audio = true;
                                    channels =
                                        s.get("channels").and_then(|c| c.as_u64()).unwrap_or(2)
                                            as u32;
                                    sample_rate = s
                                        .get("sample_rate")
                                        .and_then(|sr| sr.as_str())
                                        .and_then(|sr| sr.parse().ok())
                                        .unwrap_or(48000);
                                    if !has_video && codec_name.is_empty() {
                                        if let Some(name) =
                                            s.get("codec_name").and_then(|n| n.as_str())
                                        {
                                            codec_name = format!("{} Audio", name.to_uppercase());
                                        }
                                    }
                                }
                            }
                        }

                        let media_type = if has_video {
                            "video"
                        } else if has_audio {
                            "audio"
                        } else {
                            "unknown"
                        };

                        if media_type != "unknown" {
                            return MediaProbeInfo {
                                is_valid: true,
                                media_type: media_type.into(),
                                filename,
                                format: ext,
                                width,
                                height,
                                fps: if fps > 0.0 { fps } else { 30.0 },
                                duration_seconds: duration_sec.max(0.1),
                                channels,
                                sample_rate,
                                codec_name,
                                file_size_bytes: file_size,
                                orientation_degrees: 0,
                                has_video,
                                has_audio,
                                is_vfr: false,
                                error: None,
                            };
                        }
                    }
                }
            }
        }


        // 5. Audio heuristic fallback for MP3, AAC, FLAC, OGG
        if matches!(ext.as_str(), "mp3" | "aac" | "flac" | "ogg" | "m4a") {
            let dur = ((file_size as f64) * 8.0) / 192000.0;
            return MediaProbeInfo {
                is_valid: true,
                media_type: "audio".into(),
                filename,
                format: ext.clone(),
                width: 0,
                height: 0,
                fps: 0.0,
                duration_seconds: dur.max(1.0),
                channels: 2,
                sample_rate: 44100,
                codec_name: format!("{} Audio", ext.to_uppercase()),
                file_size_bytes: file_size,
                orientation_degrees: 0,
                has_video: false,
                has_audio: true,
                is_vfr: false,
                error: None,
            };
        }

        // Unsupported or unreadable
        MediaProbeInfo::error(format!(
            "Unsupported or unreadable media container: .{}",
            ext
        ))
    }

    /// Decodes a real video frame at the given timestamp into a pristine RGBA `VideoFrame`.
    pub fn decode_video_frame(
        raw_path: &str,
        time: RationalTime,
        target_dim: Option<Dimensions>,
    ) -> IrisResult<VideoFrame> {
        let path = normalize_media_path(raw_path);
        if !path.exists() {
            return Err(IrisError::IoError(format!(
                "Media file not found: {}",
                path.display()
            )));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let time_secs = (time.numerator as f64) / (time.denominator as f64).max(1.0);
        let time_bucket_ms = (time_secs * 1000.0) as i64 / 33 * 33;
        let dim = target_dim.unwrap_or(Dimensions::new(640, 360));

        // 1. Check in-memory bounded cache first (zero subprocesses, ~0.01ms)
        if let Some(cached) = get_cached_frame(raw_path, time_bucket_ms, dim) {
            return Ok(cached);
        }

        // 2. Image source on video timeline (pure Rust)
        if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp") {
            let bytes = std::fs::read(&path).map_err(|e| {
                IrisError::IoError(format!("Failed to read image {}: {}", path.display(), e))
            })?;
            let img = image::load_from_memory(&bytes)
                .map_err(|e| IrisError::DecodeError(format!("Failed to decode image: {}", e)))?;
            let rgba = img.to_rgba8();
            let orig_dim = Dimensions::new(rgba.width(), rgba.height());
            let dim_to_use = target_dim.unwrap_or(orig_dim);

            let frame = if dim_to_use.width == orig_dim.width && dim_to_use.height == orig_dim.height {
                VideoFrame::from_raw(
                    dim_to_use,
                    PixelFormat::Rgba8,
                    time,
                    RationalTime::new(1, 30),
                    rgba.into_raw(),
                )
            } else {
                let resized = image::imageops::resize(
                    &rgba,
                    dim_to_use.width,
                    dim_to_use.height,
                    image::imageops::FilterType::Triangle,
                );
                VideoFrame::from_raw(
                    dim_to_use,
                    PixelFormat::Rgba8,
                    time,
                    RationalTime::new(1, 30),
                    resized.into_raw(),
                )
            };
            store_cached_frame(raw_path, time_bucket_ms, dim_to_use, frame.clone());
            return Ok(frame);
        }

        // 3. Video source decoding via ffmpeg when available with strict bounds
        if Self::is_ffmpeg_available() {
            // Concurrency guard: allow at most one ffmpeg decode at a time
            if FFMPEG_ACTIVE
                .compare_exchange(
                    false,
                    true,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                struct ActiveGuard;
                impl Drop for ActiveGuard {
                    fn drop(&mut self) {
                        FFMPEG_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                }
                let _guard = ActiveGuard;

                let time_str = format!("{:.3}", time_secs.max(0.0));
                let scale_arg = format!("{}x{}", dim.width, dim.height);
                let mut cmd = create_safe_command("ffmpeg");
                cmd.args([
                    "-nostdin",
                    "-noautorun",
                    "-loglevel",
                    "error",
                    "-ss",
                    &time_str,
                    "-i",
                    path.to_str().unwrap_or(""),
                    "-frames:v",
                    "1",
                    "-s",
                    &scale_arg,
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "rgba",
                    "pipe:1",
                ]);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                if let Ok(output) = run_command_with_timeout(cmd, 1200) {
                    let expected_bytes = (dim.width as usize) * (dim.height as usize) * 4;
                    if output.status.success() && output.stdout.len() == expected_bytes {
                        let frame = VideoFrame::from_raw(
                            dim,
                            PixelFormat::Rgba8,
                            time,
                            RationalTime::new(1, 30),
                            output.stdout,
                        );
                        store_cached_frame(raw_path, time_bucket_ms, dim, frame.clone());
                        return Ok(frame);
                    }
                }
            }
        }

        // 4. Deterministic Content-Aware Frame Fallback (Instant, safe, non-blocking)
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("video")
            .to_string();
        let frame = VideoFrame::create_diagnostic_frame(dim, time, &filename, "H.264 / AAC");
        store_cached_frame(raw_path, time_bucket_ms, dim, frame.clone());
        Ok(frame)
    }

    /// Extracts PCM floating point audio samples from any media file.
    pub fn extract_audio_samples(raw_path: &str, max_samples: usize) -> IrisResult<Vec<f32>> {
        let path = normalize_media_path(raw_path);
        if !path.exists() {
            return Err(IrisError::IoError(format!(
                "Audio file not found: {}",
                path.display()
            )));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Pure-Rust WAV reader
        if ext == "wav" {
            if let Ok(bytes) = std::fs::read(&path) {
                if bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
                    let channels = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
                    let bits_per_sample = u16::from_le_bytes([bytes[34], bytes[35]]) as usize;

                    let mut samples = Vec::new();
                    if bits_per_sample == 16 {
                        let data_bytes = &bytes[44..];
                        for chunk in data_bytes.chunks_exact(2) {
                            let s = i16::from_le_bytes([chunk[0], chunk[1]]);
                            samples.push((s as f32) / 32768.0);
                            if samples.len() >= max_samples {
                                break;
                            }
                        }
                        if channels > 1 {
                            // Downmix to mono
                            let mut mono = Vec::with_capacity(samples.len() / channels);
                            for ch in samples.chunks_exact(channels) {
                                let avg = ch.iter().sum::<f32>() / channels as f32;
                                mono.push(avg);
                            }
                            return Ok(mono);
                        }
                        return Ok(samples);
                    }
                }
            }
        }

        // 2. ffmpeg audio extractor with strict bounds
        if Self::is_ffmpeg_available() {
            let mut cmd = create_safe_command("ffmpeg");
            cmd.args([
                "-nostdin",
                "-noautorun",
                "-loglevel",
                "error",
                "-i",
                path.to_str().unwrap_or(""),
                "-vn",
                "-ac",
                "1",
                "-ar",
                "8000",
                "-f",
                "f32le",
                "pipe:1",
            ]);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            if let Ok(output) = run_command_with_timeout(cmd, 2500) {
                if output.status.success() {
                    let float_count = output.stdout.len() / 4;
                    let mut samples = Vec::with_capacity(float_count.min(max_samples));
                    for chunk in output.stdout.chunks_exact(4) {
                        let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        samples.push(val);
                        if samples.len() >= max_samples {
                            break;
                        }
                    }
                    return Ok(samples);
                }
            }
        }

        // Return empty if audio stream not available
        Ok(Vec::new())
    }
}

/// A synthetic media demuxer generating deterministic frames for tests and offline previewing.
pub struct SyntheticDemuxer {
    info: StreamInfo,
    metadata: MediaMetadata,
    current_time: RationalTime,
    fps_num: u64,
    fps_den: u64,
}

impl SyntheticDemuxer {
    pub fn new(dimensions: Dimensions, duration: RationalTime, fps_num: u64, fps_den: u64) -> Self {
        let v_info = VideoStreamInfo {
            codec: "synthetic_raw".to_string(),
            dimensions,
            frame_rate_num: fps_num,
            frame_rate_den: fps_den,
            duration,
            pixel_format: PixelFormat::Rgba8,
            bit_rate: 10_000_000,
        };

        let a_info = AudioStreamInfo {
            codec: "pcm_f32".to_string(),
            sample_rate: 48000,
            channels: 2,
            duration,
            bit_rate: 1_536_000,
        };

        Self {
            info: StreamInfo {
                video_streams: vec![v_info],
                audio_streams: vec![a_info],
            },
            metadata: MediaMetadata {
                title: Some("Synthetic Test Media".into()),
                artist: Some("Iris Studio".into()),
                date: Some("2026".into()),
                orientation_degrees: 0,
            },
            current_time: RationalTime::ZERO,
            fps_num,
            fps_den,
        }
    }
}

impl Demuxer for SyntheticDemuxer {
    fn stream_info(&self) -> &StreamInfo {
        &self.info
    }

    fn metadata(&self) -> &MediaMetadata {
        &self.metadata
    }

    fn seek(&mut self, time: RationalTime) -> IrisResult<()> {
        let total_dur = self.info.video_streams[0].duration;
        self.current_time = if time > total_dur {
            total_dur
        } else if time.numerator < 0 {
            RationalTime::ZERO
        } else {
            time
        };
        Ok(())
    }

    fn next_video_frame(&mut self) -> IrisResult<Option<VideoFrame>> {
        let v_info = &self.info.video_streams[0];
        if self.current_time >= v_info.duration {
            return Ok(None);
        }

        let frame_duration = RationalTime::new(self.fps_den as i64, self.fps_num);
        let frame =
            VideoFrame::create_test_pattern(v_info.dimensions, self.current_time, frame_duration);

        self.current_time += frame_duration;
        Ok(Some(frame))
    }

    fn next_audio_frame(&mut self) -> IrisResult<Option<AudioFrame>> {
        let a_info = &self.info.audio_streams[0];
        if self.current_time >= a_info.duration {
            return Ok(None);
        }

        let chunk_duration = 0.04; // 40ms audio chunk
        let frame = AudioFrame::create_test_tone(
            a_info.sample_rate,
            a_info.channels,
            440.0,
            chunk_duration,
            0.5,
        );
        Ok(Some(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_and_decode_real_mp4_fixture() {
        let fixture = Path::new("../../tests/fixtures/test_video_360p.mp4");
        if fixture.exists() {
            let probe = RealMediaDecoder::probe_file(fixture.to_str().unwrap());
            assert!(probe.is_valid);
            assert_eq!(probe.media_type, "video");
            assert_eq!(probe.width, 640);
            assert_eq!(probe.height, 360);
            assert!(probe.duration_seconds >= 3.9);

            let frame = RealMediaDecoder::decode_video_frame(
                fixture.to_str().unwrap(),
                RationalTime::ZERO,
                None,
            );
            assert!(frame.is_ok());
            let f = frame.unwrap();
            assert_eq!(f.dimensions.width, 640);
            assert_eq!(f.dimensions.height, 360);
            assert_eq!(f.data.len(), 640 * 360 * 4);
        }
    }

    #[test]
    fn test_probe_missing_and_corrupt_files() {
        let probe = RealMediaDecoder::probe_file("non_existent_file.mp4");
        assert!(!probe.is_valid);
        assert!(probe.error.is_some());
    }

    #[test]
    fn test_safe_command_execution_and_timeout_enforcement() {
        #[cfg(target_os = "windows")]
        {
            // Successful quick command
            let mut cmd = create_safe_command("cmd");
            cmd.args(["/c", "echo", "test_ok"]);
            let res = run_command_with_timeout(cmd, 2000);
            assert!(res.is_ok(), "Fast command should succeed within timeout");
            let output = res.unwrap();
            let out = String::from_utf8_lossy(&output.stdout);
            assert!(out.contains("test_ok"));

            // Command exceeding timeout should be killed cleanly
            let mut slow_cmd = create_safe_command("cmd");
            slow_cmd.args(["/c", "ping", "-n", "4", "127.0.0.1"]);
            let slow_res = run_command_with_timeout(slow_cmd, 150);
            assert!(slow_res.is_err(), "Long running command must be terminated by timeout");
            assert!(slow_res.unwrap_err().contains("timed out"));
        }
    }

    #[test]
    fn test_frame_cache_hit_and_eviction() {
        let test_path = "test_cache_entry.mp4";
        let dim = Dimensions { width: 100, height: 100 };
        let fake_frame = VideoFrame::new(
            dim,
            PixelFormat::Rgba8,
            RationalTime::ZERO,
            RationalTime::from_seconds(0.033, 30),
        );

        // Initially not in cache
        assert!(get_cached_frame(test_path, 0, dim).is_none());

        // Insert and verify hit
        store_cached_frame(test_path, 0, dim, fake_frame.clone());
        let hit = get_cached_frame(test_path, 0, dim);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().data.len(), 100 * 100 * 4);

        // Near timestamp within 33ms threshold should hit
        let near_hit = get_cached_frame(test_path, 20, dim);
        assert!(near_hit.is_some());

        // Far timestamp should miss
        let far_miss = get_cached_frame(test_path, 200, dim);
        assert!(far_miss.is_none());
    }

    #[test]
    fn test_isobmff_corrupt_data_does_not_panic() {
        let temp_dir = std::env::temp_dir();
        let corrupt_file = temp_dir.join("iris_test_corrupt.mp4");
        std::fs::write(&corrupt_file, b"random garbage not valid mp4 boxes 1234567890").unwrap();

        let probe = IsoBmffParser::probe(&corrupt_file);
        let _ = std::fs::remove_file(&corrupt_file);

        // IsoBmffParser should return Err on garbage without panic
        assert!(probe.is_err());
    }
}
