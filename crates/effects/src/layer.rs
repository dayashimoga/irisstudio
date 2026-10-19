use crate::blend::{blend_pixel, BlendMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoLayer {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub opacity: f32, // 0.0 to 1.0
    pub blend_mode: BlendMode,
    pub visible: bool,
    pub offset_x: i32,
    pub offset_y: i32,
    pub mask: Option<Vec<u8>>, // Grayscale mask: 1 byte per pixel
}

impl PhotoLayer {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            width,
            height,
            pixels,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
            offset_x: 0,
            offset_y: 0,
            mask: None,
        }
    }
}

/// Composes a stack of photo layers onto a transparent canvas.
pub fn compose_photo_layers(canvas_w: u32, canvas_h: u32, layers: &[PhotoLayer]) -> Vec<u8> {
    let mut canvas = vec![0u8; (canvas_w * canvas_h * 4) as usize];

    for layer in layers {
        if !layer.visible || layer.opacity <= 0.0 || layer.pixels.is_empty() {
            continue;
        }

        let l_w = layer.width as i32;
        let l_h = layer.height as i32;
        let c_w = canvas_w as i32;
        let c_h = canvas_h as i32;

        let start_y = layer.offset_y.max(0);
        let end_y = (layer.offset_y + l_h).min(c_h);
        let start_x = layer.offset_x.max(0);
        let end_x = (layer.offset_x + l_w).min(c_w);

        if start_x >= end_x || start_y >= end_y {
            continue;
        }

        for y in start_y..end_y {
            let src_y = y - layer.offset_y;
            for x in start_x..end_x {
                let src_x = x - layer.offset_x;

                let src_idx = ((src_y as u32 * layer.width + src_x as u32) * 4) as usize;
                let dst_idx = ((y as u32 * canvas_w + x as u32) * 4) as usize;

                let mut layer_opacity = layer.opacity;
                if let Some(ref mask) = layer.mask {
                    let mask_idx = (src_y as u32 * layer.width + src_x as u32) as usize;
                    if mask_idx < mask.len() {
                        layer_opacity *= mask[mask_idx] as f32 / 255.0;
                    }
                }

                if layer_opacity <= 0.0 {
                    continue;
                }

                let dst_px = [
                    canvas[dst_idx],
                    canvas[dst_idx + 1],
                    canvas[dst_idx + 2],
                    canvas[dst_idx + 3],
                ];

                let src_px = [
                    layer.pixels[src_idx],
                    layer.pixels[src_idx + 1],
                    layer.pixels[src_idx + 2],
                    layer.pixels[src_idx + 3],
                ];

                let blended = blend_pixel(dst_px, src_px, layer.blend_mode, layer_opacity);
                canvas[dst_idx..dst_idx + 4].copy_from_slice(&blended);
            }
        }
    }

    canvas
}
