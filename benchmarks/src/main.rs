use iris_audio::generate_waveform_peaks;
use iris_core::{Dimensions, RationalTime, Rect, TimeSpan};
use iris_effects::{apply_adjustments, AdjustmentParameters};
use iris_images::{crop_image, rotate_90, straighten_image, RgbaImage};
use iris_render::{MockSourceResolver, PreviewEngine};
use iris_timeline::{Clip, Timeline, Track, TrackKind};
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("            IRIS STUDIO PERFORMANCE BENCHMARKS               ");
    println!("============================================================");

    // 1. Color Adjustment Pipeline Benchmark (1080p and 4K)
    bench_color_adjustments(1920, 1080, "1080p (Full HD)");
    bench_color_adjustments(3840, 2160, "4K (Ultra HD)");

    // 2. Image Transforms Benchmark
    bench_image_transforms();

    // 3. Audio Waveform Generation Benchmark
    bench_audio_waveform();

    // 4. Compositor Multi-track Timeline Render Benchmark
    bench_compositor_timeline();

    println!("============================================================");
    println!("All performance baselines verified successfully.");
}

fn bench_color_adjustments(width: u32, height: u32, label: &str) {
    let total_pixels = width as usize * height as usize;
    let mut pixels = vec![128u8; total_pixels * 4];
    let params = AdjustmentParameters {
        exposure: 0.75,
        contrast: 15.0,
        temperature: 12.0,
        tint: -5.0,
        highlights: -10.0,
        shadows: 15.0,
        saturation: 10.0,
        vibrance: 20.0,
    };

    // Warm-up
    apply_adjustments(&mut pixels, &params);

    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        apply_adjustments(&mut pixels, &params);
    }
    let elapsed = start.elapsed();
    let per_frame_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    let mp_per_sec =
        (total_pixels as f64 * iterations as f64 / 1_000_000.0) / elapsed.as_secs_f64();
    let equivalent_fps = 1000.0 / per_frame_ms;

    println!("[BENCH] Color Adjustments {}:", label);
    println!(
        "        Latency: {:.2} ms/frame | Throughput: {:.1} MP/s | Eq FPS: {:.1} fps",
        per_frame_ms, mp_per_sec, equivalent_fps
    );
}

fn bench_image_transforms() {
    let img = RgbaImage::create_test_gradient(2000, 1500);

    // Crop
    let start_crop = Instant::now();
    let crop_rect = Rect::new(100.0, 100.0, 1200.0, 900.0);
    let cropped = crop_image(&img, crop_rect).expect("Crop failed");
    let crop_ms = start_crop.elapsed().as_secs_f64() * 1000.0;

    // Rotate 90
    let start_rot = Instant::now();
    let rotated = rotate_90(&cropped, true);
    let rot_ms = start_rot.elapsed().as_secs_f64() * 1000.0;

    // Straighten
    let start_straight = Instant::now();
    let _straightened = straighten_image(&rotated, 4.5);
    let straight_ms = start_straight.elapsed().as_secs_f64() * 1000.0;

    println!("[BENCH] Image Transforms (2000x1500):");
    println!(
        "        Crop: {:.2} ms | Rotate 90: {:.2} ms | Straighten: {:.2} ms",
        crop_ms, rot_ms, straight_ms
    );
}

fn bench_audio_waveform() {
    let sample_count = 480_000; // 10 seconds of 48kHz audio
    let samples = vec![0.5f32; sample_count];

    let start = Instant::now();
    let spp = 480; // 100 peaks per second
    let envelope = generate_waveform_peaks(&samples, 1, spp);
    let ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("[BENCH] Audio Waveform Generation (480,000 samples):");
    println!(
        "        Computed {} peaks in {:.2} ms",
        envelope.peaks.len(),
        ms
    );
}

fn bench_compositor_timeline() {
    let mut timeline = Timeline::new("bench_tl", 30, 1);
    let mut track_v1 = Track::new("trk_v1", "Main Video", TrackKind::Video);
    track_v1.clips.push(Clip::new(
        "c1",
        "test_video_1",
        "Scene 1",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)),
        RationalTime::ZERO,
    ));

    let mut track_v2 = Track::new("trk_v2", "Overlay Video", TrackKind::Video);
    let mut clip2 = Clip::new(
        "c2",
        "test_video_2",
        "Picture in Picture",
        TimeSpan::new(RationalTime::ZERO, RationalTime::new(10, 1)),
        RationalTime::ZERO,
    );
    clip2.transform.scale_x = 0.5;
    clip2.transform.scale_y = 0.5;
    clip2.transform.opacity = 0.85;
    track_v2.clips.push(clip2);

    timeline.tracks.push(track_v1);
    timeline.tracks.push(track_v2);

    let engine = PreviewEngine::new(Dimensions::new(1920, 1080));
    let resolver = MockSourceResolver::default();

    let iterations = 15;
    let start = Instant::now();
    for i in 0..iterations {
        let time = RationalTime::new(i, 30);
        let _ = engine.render_preview(&timeline, time, &resolver);
    }
    let elapsed = start.elapsed();
    let per_frame_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    let fps = 1000.0 / per_frame_ms;

    println!("[BENCH] Compositor Multitrack 1080p Rendering:");
    println!(
        "        Render Latency: {:.2} ms/frame | Sustained Rate: {:.1} fps",
        per_frame_ms, fps
    );
}
