use crate::buffer::RgbaImage;
use iris_core::{IrisError, IrisResult};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
}

pub fn decode_image(bytes: &[u8]) -> IrisResult<RgbaImage> {
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| IrisError::DecodeError(e.to_string()))?;

    let dynamic_img = reader
        .decode()
        .map_err(|e| IrisError::DecodeError(e.to_string()))?;

    let rgba = dynamic_img.to_rgba8();
    let (width, height) = rgba.dimensions();

    RgbaImage::from_raw(width, height, rgba.into_raw())
}

pub fn encode_png(img: &RgbaImage) -> IrisResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut bytes));
    image::ImageEncoder::write_image(
        encoder,
        &img.pixels,
        img.width,
        img.height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| IrisError::EncodeError(e.to_string()))?;

    Ok(bytes)
}

pub fn encode_jpeg(img: &RgbaImage, quality: u8) -> IrisResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
        Cursor::new(&mut bytes),
        quality.clamp(1, 100),
    );

    // Convert RGBA to RGB for JPEG
    let mut rgb_pixels = Vec::with_capacity((img.width * img.height * 3) as usize);
    for chunk in img.pixels.chunks_exact(4) {
        rgb_pixels.push(chunk[0]);
        rgb_pixels.push(chunk[1]);
        rgb_pixels.push(chunk[2]);
    }

    image::ImageEncoder::write_image(
        encoder,
        &rgb_pixels,
        img.width,
        img.height,
        image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| IrisError::EncodeError(e.to_string()))?;

    Ok(bytes)
}

pub fn encode_webp(img: &RgbaImage) -> IrisResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(Cursor::new(&mut bytes));
    image::ImageEncoder::write_image(
        encoder,
        &img.pixels,
        img.width,
        img.height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| IrisError::EncodeError(e.to_string()))?;

    Ok(bytes)
}
