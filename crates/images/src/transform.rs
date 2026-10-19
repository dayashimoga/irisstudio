use crate::buffer::RgbaImage;
use iris_core::{IrisError, IrisResult, Rect};

pub fn crop_image(img: &RgbaImage, rect: Rect) -> IrisResult<RgbaImage> {
    let x_start = (rect.x.max(0.0)).round() as u32;
    let y_start = (rect.y.max(0.0)).round() as u32;
    let width = (rect.width.max(1.0)).round() as u32;
    let height = (rect.height.max(1.0)).round() as u32;

    if x_start + width > img.width || y_start + height > img.height {
        return Err(IrisError::InvalidParameter(format!(
            "Crop rectangle {:?} out of image bounds ({}x{})",
            rect, img.width, img.height
        )));
    }

    let mut cropped = RgbaImage::new(width, height);
    for y in 0..height {
        let src_y = y_start + y;
        let src_offset = ((src_y * img.width + x_start) * 4) as usize;
        let src_len = (width * 4) as usize;
        let dst_offset = ((y * width) * 4) as usize;

        cropped.pixels[dst_offset..dst_offset + src_len]
            .copy_from_slice(&img.pixels[src_offset..src_offset + src_len]);
    }

    Ok(cropped)
}

pub fn rotate_90(img: &RgbaImage, clockwise: bool) -> RgbaImage {
    let mut rotated = RgbaImage::new(img.height, img.width);
    for y in 0..img.height {
        for x in 0..img.width {
            let px = img.get_pixel(x, y);
            if clockwise {
                rotated.set_pixel(img.height - 1 - y, x, px);
            } else {
                rotated.set_pixel(y, img.width - 1 - x, px);
            }
        }
    }
    rotated
}

pub fn flip_horizontal(img: &RgbaImage) -> RgbaImage {
    let mut flipped = RgbaImage::new(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let px = img.get_pixel(x, y);
            flipped.set_pixel(img.width - 1 - x, y, px);
        }
    }
    flipped
}

pub fn flip_vertical(img: &RgbaImage) -> RgbaImage {
    let mut flipped = RgbaImage::new(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let px = img.get_pixel(x, y);
            flipped.set_pixel(x, img.height - 1 - y, px);
        }
    }
    flipped
}

pub fn resize_bilinear(img: &RgbaImage, new_width: u32, new_height: u32) -> RgbaImage {
    assert!(new_width > 0 && new_height > 0);
    let mut resized = RgbaImage::new(new_width, new_height);

    let x_ratio = (img.width - 1) as f32 / new_width as f32;
    let y_ratio = (img.height - 1) as f32 / new_height as f32;

    for y in 0..new_height {
        let y_l = (y_ratio * y as f32).floor() as u32;
        let y_h = (y_l + 1).min(img.height - 1);
        let y_weight = (y_ratio * y as f32) - y_l as f32;

        for x in 0..new_width {
            let x_l = (x_ratio * x as f32).floor() as u32;
            let x_h = (x_l + 1).min(img.width - 1);
            let x_weight = (x_ratio * x as f32) - x_l as f32;

            let a = img.get_pixel(x_l, y_l);
            let b = img.get_pixel(x_h, y_l);
            let c = img.get_pixel(x_l, y_h);
            let d = img.get_pixel(x_h, y_h);

            let mut out = [0u8; 4];
            for i in 0..4 {
                let top = (a[i] as f32) * (1.0 - x_weight) + (b[i] as f32) * x_weight;
                let bottom = (c[i] as f32) * (1.0 - x_weight) + (d[i] as f32) * x_weight;
                let val = top * (1.0 - y_weight) + bottom * y_weight;
                out[i] = val.round().clamp(0.0, 255.0) as u8;
            }
            resized.set_pixel(x, y, out);
        }
    }

    resized
}

pub fn straighten_image(img: &RgbaImage, angle_degrees: f32) -> RgbaImage {
    let rad = -angle_degrees.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();

    let cx = img.width as f32 / 2.0;
    let cy = img.height as f32 / 2.0;

    let mut out = RgbaImage::new(img.width, img.height);

    for y in 0..img.height {
        let dy = y as f32 - cy;
        for x in 0..img.width {
            let dx = x as f32 - cx;

            // Backward mapping
            let src_x = cx + (dx * cos_a - dy * sin_a);
            let src_y = cy + (dx * sin_a + dy * cos_a);

            if src_x >= 0.0
                && src_x < (img.width - 1) as f32
                && src_y >= 0.0
                && src_y < (img.height - 1) as f32
            {
                let x0 = src_x.floor() as u32;
                let y0 = src_y.floor() as u32;
                let x1 = x0 + 1;
                let y1 = y0 + 1;

                let wx = src_x - x0 as f32;
                let wy = src_y - y0 as f32;

                let p00 = img.get_pixel(x0, y0);
                let p10 = img.get_pixel(x1, y0);
                let p01 = img.get_pixel(x0, y1);
                let p11 = img.get_pixel(x1, y1);

                let mut col = [0u8; 4];
                for c in 0..4 {
                    let top = p00[c] as f32 * (1.0 - wx) + p10[c] as f32 * wx;
                    let bot = p01[c] as f32 * (1.0 - wx) + p11[c] as f32 * wx;
                    col[c] = (top * (1.0 - wy) + bot * wy).clamp(0.0, 255.0) as u8;
                }
                out.set_pixel(x, y, col);
            } else {
                out.set_pixel(x, y, [0, 0, 0, 0]); // Transparent black boundary
            }
        }
    }

    out
}
