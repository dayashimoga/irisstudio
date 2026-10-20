use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplinePoint {
    pub x: f32, // Normalized 0.0 - 1.0
    pub y: f32, // Normalized 0.0 - 1.0
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
    /// Generates a 256-entry lookup table from control points using Fritsch-Carlson monotonic cubic spline.
    pub fn from_points(mut points: Vec<SplinePoint>) -> Self {
        if points.is_empty() {
            return Self::default();
        }

        // Sort by x coordinate
        points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate control points with nearly identical x coordinates
        let mut clean_points: Vec<SplinePoint> = Vec::with_capacity(points.len() + 2);
        for pt in points {
            if let Some(last) = clean_points.last() {
                if (pt.x - last.x).abs() < 1e-4 {
                    continue;
                }
            }
            clean_points.push(pt);
        }

        // Ensure boundary endpoints at (0.0, ...) and (1.0, ...)
        if clean_points.is_empty() {
            return Self::default();
        }
        if clean_points.first().unwrap().x > 0.0 {
            clean_points.insert(0, SplinePoint { x: 0.0, y: 0.0 });
        }
        if clean_points.last().unwrap().x < 1.0 {
            clean_points.push(SplinePoint { x: 1.0, y: 1.0 });
        }

        let n = clean_points.len();
        if n < 2 {
            return Self::default();
        }

        // 1. Calculate secants (delta_k)
        let mut deltas = vec![0.0f32; n - 1];
        let mut h = vec![0.0f32; n - 1];
        for k in 0..n - 1 {
            h[k] = (clean_points[k + 1].x - clean_points[k].x).max(1e-6);
            deltas[k] = (clean_points[k + 1].y - clean_points[k].y) / h[k];
        }

        // 2. Initialize tangents (d_k)
        let mut d = vec![0.0f32; n];
        d[0] = deltas[0];
        d[n - 1] = deltas[n - 2];
        for k in 1..n - 1 {
            d[k] = (deltas[k - 1] + deltas[k]) * 0.5;
        }

        // 3. Fritsch-Carlson monotonicity check & adjustment
        for k in 0..n - 1 {
            if deltas[k].abs() < 1e-7 {
                d[k] = 0.0;
                d[k + 1] = 0.0;
            } else {
                let alpha = d[k] / deltas[k];
                let beta = d[k + 1] / deltas[k];

                if alpha < 0.0 {
                    d[k] = 0.0;
                }
                if beta < 0.0 {
                    d[k + 1] = 0.0;
                }

                let sum_sq = alpha * alpha + beta * beta;
                if sum_sq > 9.0 {
                    let tau = 3.0 / sum_sq.sqrt();
                    d[k] = tau * alpha * deltas[k];
                    d[k + 1] = tau * beta * deltas[k];
                }
            }
        }

        // 4. Sample the monotonic cubic Hermite spline across 256 LUT bins
        let mut lut = vec![0u8; 256];
        for (i, slot) in lut.iter_mut().enumerate() {
            let x = i as f32 / 255.0;

            // Locate segment
            let mut seg = 0;
            while seg < n - 2 && clean_points[seg + 1].x < x {
                seg += 1;
            }

            let p0 = clean_points[seg];
            let p1 = clean_points[seg + 1];
            let seg_h = h[seg];
            let t = ((x - p0.x) / seg_h).clamp(0.0, 1.0);

            // Hermite basis functions
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;

            let y = h00 * p0.y + h10 * seg_h * d[seg] + h01 * p1.y + h11 * seg_h * d[seg + 1];

            *slot = (y * 255.0).round().clamp(0.0, 255.0) as u8;
        }

        Self { lut }
    }

    #[inline(always)]
    pub fn map(&self, val: u8) -> u8 {
        self.lut[val as usize]
    }

    pub fn apply(&self, pixels: &mut [u8]) {
        for chunk in pixels.as_chunks_mut::<4>().0 {
            chunk[0] = self.lut[chunk[0] as usize];
            chunk[1] = self.lut[chunk[1] as usize];
            chunk[2] = self.lut[chunk[2] as usize];
        }
    }
}

/// Parametric multi-channel RGB curves representing Master, Red, Green, and Blue channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RgbCurves {
    pub master: CurveLut,
    pub red: CurveLut,
    pub green: CurveLut,
    pub blue: CurveLut,
}

impl RgbCurves {
    pub fn new(
        master_points: Vec<SplinePoint>,
        red_points: Vec<SplinePoint>,
        green_points: Vec<SplinePoint>,
        blue_points: Vec<SplinePoint>,
    ) -> Self {
        Self {
            master: CurveLut::from_points(master_points),
            red: CurveLut::from_points(red_points),
            green: CurveLut::from_points(green_points),
            blue: CurveLut::from_points(blue_points),
        }
    }

    /// Evaluates all four channels in place across an RGBA pixel buffer.
    pub fn apply_rgba(&self, pixels: &mut [u8]) {
        let m = &self.master.lut;
        let r_lut = &self.red.lut;
        let g_lut = &self.green.lut;
        let b_lut = &self.blue.lut;

        for chunk in pixels.as_chunks_mut::<4>().0 {
            let r = r_lut[chunk[0] as usize];
            let g = g_lut[chunk[1] as usize];
            let b = b_lut[chunk[2] as usize];

            chunk[0] = m[r as usize];
            chunk[1] = m[g as usize];
            chunk[2] = m[b as usize];
        }
    }
}
