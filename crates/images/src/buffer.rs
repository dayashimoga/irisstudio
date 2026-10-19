use iris_core::{Dimensions, IrisError, IrisResult, PixelFormat};

#[derive(Debug, Clone, PartialEq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA, 4 bytes per pixel
}

impl RgbaImage {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "Dimensions must be positive");
        let total = width as usize * height as usize * 4;
        Self {
            width,
            height,
            pixels: vec![0u8; total],
        }
    }

    pub fn from_raw(width: u32, height: u32, pixels: Vec<u8>) -> IrisResult<Self> {
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            return Err(IrisError::InvalidParameter(format!(
                "Pixel buffer size mismatch: expected {} bytes, got {}",
                expected,
                pixels.len()
            )));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn dimensions(&self) -> Dimensions {
        Dimensions::new(self.width, self.height)
    }

    pub fn pixel_format(&self) -> PixelFormat {
        PixelFormat::Rgba8
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = (y as usize * self.width as usize + x as usize) * 4;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let idx = (y as usize * self.width as usize + x as usize) * 4;
        self.pixels[idx] = color[0];
        self.pixels[idx + 1] = color[1];
        self.pixels[idx + 2] = color[2];
        self.pixels[idx + 3] = color[3];
    }

    /// Creates a test gradient image for testing and verification.
    pub fn create_test_gradient(width: u32, height: u32) -> Self {
        let mut img = Self::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as u8;
                let g = ((y as f32 / height as f32) * 255.0) as u8;
                let b = (128.0 + (x as f32 - y as f32) * 0.1).clamp(0.0, 255.0) as u8;
                img.set_pixel(x, y, [r, g, b, 255]);
            }
        }
        img
    }
}
