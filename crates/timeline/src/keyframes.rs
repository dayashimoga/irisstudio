//! Keyframe interpolation and easing engine.

use iris_core::RationalTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32), // (x1, y1, x2, y2)
}

impl Easing {
    /// Evaluates the easing function for normalized progress u in [0.0, 1.0].
    pub fn evaluate(&self, u: f32) -> f32 {
        let u = u.clamp(0.0, 1.0);
        match self {
            Self::Linear => u,
            Self::EaseIn => u * u,
            Self::EaseOut => 1.0 - (1.0 - u) * (1.0 - u),
            Self::EaseInOut => {
                if u < 0.5 {
                    2.0 * u * u
                } else {
                    1.0 - (-2.0 * u + 2.0).powi(2) / 2.0
                }
            }
            Self::CubicBezier(x1, y1, x2, y2) => solve_cubic_bezier(u, *x1, *y1, *x2, *y2),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: RationalTime,
    pub value: f32,
    #[serde(default)]
    pub easing: Easing,
}

impl Keyframe {
    pub fn new(time: RationalTime, value: f32) -> Self {
        Self {
            time,
            value,
            easing: Easing::Linear,
        }
    }

    pub fn with_easing(time: RationalTime, value: f32, easing: Easing) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// Evaluates a timeline parameter given a set of ordered keyframes and target time.
pub fn interpolate_keyframes(keyframes: &[Keyframe], time: RationalTime, default_val: f32) -> f32 {
    if keyframes.is_empty() {
        return default_val;
    }
    if keyframes.len() == 1 {
        return keyframes[0].value;
    }

    if time <= keyframes[0].time {
        return keyframes[0].value;
    }
    if time >= keyframes.last().unwrap().time {
        return keyframes.last().unwrap().value;
    }

    for i in 0..keyframes.len() - 1 {
        let k0 = &keyframes[i];
        let k1 = &keyframes[i + 1];

        if time >= k0.time && time <= k1.time {
            let t0 = k0.time.to_seconds();
            let t1 = k1.time.to_seconds();
            let cur = time.to_seconds();
            let duration = (t1 - t0).max(1e-6);
            let raw_factor = ((cur - t0) / duration) as f32;

            let eased_factor = k0.easing.evaluate(raw_factor);
            return k0.value + (k1.value - k0.value) * eased_factor;
        }
    }

    default_val
}

/// Solves a cubic bezier curve given target X (u) and control points (x1, y1, x2, y2).
fn solve_cubic_bezier(u: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }

    // Binary search for t such that sample_bezier_x(t) == u
    let mut t_low = 0.0f32;
    let mut t_high = 1.0f32;
    let mut t = u;

    for _ in 0..12 {
        let current_x = sample_bezier(t, x1, x2);
        if (current_x - u).abs() < 1e-5 {
            break;
        }
        if current_x < u {
            t_low = t;
        } else {
            t_high = t;
        }
        t = (t_low + t_high) * 0.5;
    }

    sample_bezier(t, y1, y2).clamp(0.0, 1.0)
}

#[inline]
fn sample_bezier(t: f32, p1: f32, p2: f32) -> f32 {
    // 3*(1-t)^2 * t * p1 + 3*(1-t) * t^2 * p2 + t^3
    let inv_t = 1.0 - t;
    3.0 * inv_t * inv_t * t * p1 + 3.0 * inv_t * t * t * p2 + t * t * t
}
