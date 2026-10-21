use iris_core::{Dimensions, PixelFormat, RationalTime};

#[derive(Debug, Clone, PartialEq)]
pub struct VideoFrame {
    pub dimensions: Dimensions,
    pub format: PixelFormat,
    pub timestamp: RationalTime,
    pub duration: RationalTime,
    pub stride: usize,
    pub data: Vec<u8>,
}

impl VideoFrame {
    pub fn new(
        dimensions: Dimensions,
        format: PixelFormat,
        timestamp: RationalTime,
        duration: RationalTime,
    ) -> Self {
        let stride = dimensions.width as usize * format.bytes_per_pixel();
        let total_size = stride * dimensions.height as usize;
        Self {
            dimensions,
            format,
            timestamp,
            duration,
            stride,
            data: vec![0u8; total_size],
        }
    }

    pub fn from_raw(
        dimensions: Dimensions,
        format: PixelFormat,
        timestamp: RationalTime,
        duration: RationalTime,
        data: Vec<u8>,
    ) -> Self {
        let stride = dimensions.width as usize * format.bytes_per_pixel();
        assert_eq!(
            data.len(),
            stride * dimensions.height as usize,
            "VideoFrame data size must match width * height * bytes_per_pixel"
        );
        Self {
            dimensions,
            format,
            timestamp,
            duration,
            stride,
            data,
        }
    }

    /// Generates a deterministic SMPTE-style test pattern for verification & testing.
    pub fn create_test_pattern(
        dimensions: Dimensions,
        timestamp: RationalTime,
        duration: RationalTime,
    ) -> Self {
        let mut frame = Self::new(dimensions, PixelFormat::Rgba8, timestamp, duration);
        let w = dimensions.width as usize;
        let h = dimensions.height as usize;

        // 7 standard primary/secondary colors: White, Yellow, Cyan, Green, Magenta, Red, Blue
        let colors: [[u8; 4]; 7] = [
            [235, 235, 235, 255], // White
            [219, 219, 16, 255],  // Yellow
            [16, 219, 219, 255],  // Cyan
            [16, 219, 16, 255],   // Green
            [219, 16, 219, 255],  // Magenta
            [219, 16, 16, 255],   // Red
            [16, 16, 219, 255],   // Blue
        ];

        let bar_width = w / colors.len();

        for y in 0..h {
            for x in 0..w {
                let color_idx = (x / bar_width.max(1)).min(colors.len() - 1);
                let col = colors[color_idx];
                let offset = (y * w + x) * 4;
                frame.data[offset] = col[0];
                frame.data[offset + 1] = col[1];
                frame.data[offset + 2] = col[2];
                frame.data[offset + 3] = col[3];
            }
        }

        frame
    }

    /// Creates an empty studio background frame (clean dark neutral slate).
    pub fn create_empty_frame(dimensions: Dimensions) -> Self {
        let mut frame = Self::new(
            dimensions,
            PixelFormat::Rgba8,
            RationalTime::ZERO,
            RationalTime::new(1, 30),
        );
        for chunk in frame.data.as_chunks_mut::<4>().0 {
            chunk[0] = 18;
            chunk[1] = 22;
            chunk[2] = 30;
            chunk[3] = 255;
        }
        frame
    }

    /// Creates a deterministic content-aware diagnostic preview frame when external decoder is absent.
    pub fn create_diagnostic_frame(
        dimensions: Dimensions,
        timestamp: RationalTime,
        filename: &str,
        codec: &str,
    ) -> Self {
        let mut frame = Self::new(
            dimensions,
            PixelFormat::Rgba8,
            timestamp,
            RationalTime::new(1, 30),
        );
        let w = dimensions.width as usize;
        let h = dimensions.height as usize;

        let hash = filename
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let base_r = ((hash & 0xFF) as u8).clamp(30, 180);
        let base_g = (((hash >> 8) & 0xFF) as u8).clamp(30, 180);
        let base_b = (((hash >> 16) & 0xFF) as u8).clamp(40, 200);

        let t_secs = (timestamp.numerator as f64) / (timestamp.denominator as f64).max(1.0);
        let sweep = ((t_secs * 60.0) as usize) % w.max(1);

        for y in 0..h {
            let row_factor = ((y * 255) / h.max(1)) as u8;
            for x in 0..w {
                let offset = (y * w + x) * 4;
                let is_sweep = (x as isize - sweep as isize).abs() < 4;
                if is_sweep {
                    frame.data[offset] = 0;
                    frame.data[offset + 1] = 229;
                    frame.data[offset + 2] = 255;
                    frame.data[offset + 3] = 255;
                } else {
                    let r = (base_r / 2).saturating_add(row_factor / 4);
                    let g = (base_g / 2).saturating_add(row_factor / 4);
                    let b = (base_b / 2).saturating_add(row_factor / 3);
                    frame.data[offset] = r;
                    frame.data[offset + 1] = g;
                    frame.data[offset + 2] = b;
                    frame.data[offset + 3] = 255;
                }
            }
        }
        let _ = codec;
        frame
    }

    pub fn byte_size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioFrame {
    pub sample_rate: u32,
    pub channels: u16,
    pub timestamp: RationalTime,
    pub duration: RationalTime,
    pub samples: Vec<f32>,
}

impl AudioFrame {
    pub fn new(
        sample_rate: u32,
        channels: u16,
        timestamp: RationalTime,
        samples: Vec<f32>,
    ) -> Self {
        let num_samples_per_channel = samples.len() / channels as usize;
        let duration = RationalTime::new(num_samples_per_channel as i64, sample_rate as u64);
        Self {
            sample_rate,
            channels,
            timestamp,
            duration,
            samples,
        }
    }

    /// Generates a synthetic sine wave audio tone for tests.
    pub fn create_test_tone(
        sample_rate: u32,
        channels: u16,
        frequency_hz: f32,
        duration_seconds: f64,
        amplitude: f32,
    ) -> Self {
        let total_samples = (sample_rate as f64 * duration_seconds).round() as usize;
        let mut samples = Vec::with_capacity(total_samples * channels as usize);
        let step = (2.0 * std::f32::consts::PI * frequency_hz) / sample_rate as f32;

        for i in 0..total_samples {
            let val = (i as f32 * step).sin() * amplitude;
            for _ in 0..channels {
                samples.push(val);
            }
        }

        let duration = RationalTime::from_seconds(duration_seconds, sample_rate as u64);
        Self {
            sample_rate,
            channels,
            timestamp: RationalTime::ZERO,
            duration,
            samples,
        }
    }
}
