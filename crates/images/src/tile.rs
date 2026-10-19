use crate::buffer::RgbaImage;
use iris_core::{Dimensions, Rect};

#[derive(Debug, Clone)]
pub struct ImageTile {
    pub col: u32,
    pub row: u32,
    pub bounds: Rect,
    pub data: RgbaImage,
}

pub struct TiledImage {
    pub total_dimensions: Dimensions,
    pub tile_size: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
}

impl TiledImage {
    pub const DEFAULT_TILE_SIZE: u32 = 512;

    pub fn new(total_dimensions: Dimensions, tile_size: u32) -> Self {
        let tiles_x = total_dimensions.width.div_ceil(tile_size);
        let tiles_y = total_dimensions.height.div_ceil(tile_size);
        Self {
            total_dimensions,
            tile_size,
            tiles_x,
            tiles_y,
        }
    }

    pub fn tile_bounds(&self, col: u32, row: u32) -> Rect {
        let x = (col * self.tile_size) as f32;
        let y = (row * self.tile_size) as f32;
        let w = (self.tile_size as f32).min(self.total_dimensions.width as f32 - x);
        let h = (self.tile_size as f32).min(self.total_dimensions.height as f32 - y);
        Rect::new(x, y, w, h)
    }

    /// Extracts a specific tile from a full image buffer.
    pub fn extract_tile(&self, source: &RgbaImage, col: u32, row: u32) -> Option<ImageTile> {
        if col >= self.tiles_x || row >= self.tiles_y {
            return None;
        }
        let bounds = self.tile_bounds(col, row);
        let tw = bounds.width as u32;
        let th = bounds.height as u32;

        let mut tile_img = RgbaImage::new(tw, th);
        for y in 0..th {
            let src_y = (bounds.y as u32) + y;
            let src_x = bounds.x as u32;
            let src_offset = ((src_y * source.width + src_x) * 4) as usize;
            let dst_offset = ((y * tw) * 4) as usize;
            let len = (tw * 4) as usize;

            tile_img.pixels[dst_offset..dst_offset + len]
                .copy_from_slice(&source.pixels[src_offset..src_offset + len]);
        }

        Some(ImageTile {
            col,
            row,
            bounds,
            data: tile_img,
        })
    }
}
