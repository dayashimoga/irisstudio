use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplinePoint {
    pub x: f32, // 0.0 - 1.0
    pub y: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurveLut {
    pub lut: Vec<u8>,
}

impl Default for CurveLut {
    fn default() -> Self {
        Self {
            lut: (0..=255).collect(),
        }
    }
}

impl CurveLut {
    pub fn from_points(mut points: Vec<SplinePoint>) -> Self {
        if points.is_empty() {
            return Self::default();
        }

        // Sort by x
        points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        // Ensure start (0,0) and end (1,1) if missing
        if points.first().unwrap().x > 0.0 {
            points.insert(0, SplinePoint { x: 0.0, y: 0.0 });
        }
        if points.last().unwrap().x < 1.0 {
            points.push(SplinePoint { x: 1.0, y: 1.0 });
        }

        let mut lut = vec![0u8; 256];

        for (i, slot) in lut.iter_mut().enumerate() {
            let x = i as f32 / 255.0;

            // Find segment
            let mut seg = 0;
            while seg < points.len() - 1 && points[seg + 1].x < x {
                seg += 1;
            }

            if seg >= points.len() - 1 {
                *slot = (points.last().unwrap().y * 255.0).round().clamp(0.0, 255.0) as u8;
                continue;
            }

            let p0 = points[seg];
            let p1 = points[seg + 1];
            let t = if p1.x == p0.x {
                0.0
            } else {
                (x - p0.x) / (p1.x - p0.x)
            };

            // Smoothstep curve interpolation
            let smooth_t = t * t * (3.0 - 2.0 * t);
            let y = p0.y + (p1.y - p0.y) * smooth_t;

            *slot = (y * 255.0).round().clamp(0.0, 255.0) as u8;
        }

        Self { lut }
    }

    #[inline]
    pub fn map(&self, val: u8) -> u8 {
        self.lut[val as usize]
    }

    pub fn apply(&self, pixels: &mut [u8]) {
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[0] = self.lut[chunk[0] as usize];
            chunk[1] = self.lut[chunk[1] as usize];
            chunk[2] = self.lut[chunk[2] as usize];
        }
    }
}
