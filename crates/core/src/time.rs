use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Exact rational time representation to avoid floating-point drift.
/// Represents a time as `numerator / denominator` in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RationalTime {
    pub numerator: i64,
    pub denominator: u64,
}

impl RationalTime {
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    pub fn new(numerator: i64, denominator: u64) -> Self {
        if denominator == 0 {
            panic!("RationalTime denominator cannot be zero");
        }
        let mut r = Self {
            numerator,
            denominator,
        };
        r.reduce();
        r
    }

    pub fn from_seconds(seconds: f64, timebase: u64) -> Self {
        let num = (seconds * timebase as f64).round() as i64;
        Self::new(num, timebase)
    }

    pub fn from_frames(frame_index: i64, fps_num: u64, fps_den: u64) -> Self {
        // Time = frame_index * (fps_den / fps_num)
        Self::new(frame_index * (fps_den as i64), fps_num)
    }

    pub fn to_seconds(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    pub fn to_frames(self, fps_num: u64, fps_den: u64) -> i64 {
        // frame = (num / den) / (fps_den / fps_num) = (num * fps_num) / (den * fps_den)
        let total_num = self.numerator as i128 * fps_num as i128;
        let total_den = self.denominator as i128 * fps_den as i128;
        if total_den == 0 {
            0
        } else {
            (total_num / total_den) as i64
        }
    }

    pub fn rescale(self, new_denominator: u64) -> Self {
        if self.denominator == new_denominator {
            return self;
        }
        let scaled_num =
            (self.numerator as i128 * new_denominator as i128) / self.denominator as i128;
        Self {
            numerator: scaled_num as i64,
            denominator: new_denominator,
        }
    }

    fn reduce(&mut self) {
        if self.numerator == 0 {
            self.denominator = 1;
            return;
        }
        let g = gcd(self.numerator.unsigned_abs(), self.denominator);
        if g > 1 {
            self.numerator /= g as i64;
            self.denominator /= g;
        }
    }

    pub fn to_timecode(self) -> String {
        let total_secs = self.to_seconds().max(0.0);
        let hours = (total_secs / 3600.0).floor() as u32;
        let rem1 = total_secs - (hours as f64 * 3600.0);
        let minutes = (rem1 / 60.0).floor() as u32;
        let rem2 = rem1 - (minutes as f64 * 60.0);
        let seconds = rem2.floor() as u32;
        let millis = ((rem2 - seconds as f64) * 1000.0).round() as u32;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.numerator as i128 * other.denominator as i128;
        let b = other.numerator as i128 * self.denominator as i128;
        a.cmp(&b)
    }
}

impl Add for RationalTime {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let num = self.numerator as i128 * other.denominator as i128
            + other.numerator as i128 * self.denominator as i128;
        let den = self.denominator as i128 * other.denominator as i128;
        let g = gcd(num.unsigned_abs() as u64, den as u64);
        Self {
            numerator: (num / g as i128) as i64,
            denominator: (den / g as i128) as u64,
        }
    }
}

impl AddAssign for RationalTime {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for RationalTime {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let num = self.numerator as i128 * other.denominator as i128
            - other.numerator as i128 * self.denominator as i128;
        let den = self.denominator as i128 * other.denominator as i128;
        let g = gcd(num.unsigned_abs() as u64, den as u64);
        Self {
            numerator: (num / g as i128) as i64,
            denominator: (den / g as i128) as u64,
        }
    }
}

impl SubAssign for RationalTime {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl fmt::Display for RationalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}s ({:.3}s)",
            self.numerator,
            self.denominator,
            self.to_seconds()
        )
    }
}

/// A continuous time range bounded by start time and duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeSpan {
    pub start: RationalTime,
    pub duration: RationalTime,
}

impl TimeSpan {
    pub fn new(start: RationalTime, duration: RationalTime) -> Self {
        assert!(
            duration.numerator >= 0,
            "TimeSpan duration cannot be negative"
        );
        Self { start, duration }
    }

    pub fn end(self) -> RationalTime {
        self.start + self.duration
    }

    pub fn contains(self, time: RationalTime) -> bool {
        time >= self.start && time < self.end()
    }

    pub fn contains_inclusive(self, time: RationalTime) -> bool {
        time >= self.start && time <= self.end()
    }

    pub fn intersects(self, other: Self) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    pub fn clamp(self, time: RationalTime) -> RationalTime {
        if time < self.start {
            self.start
        } else if time > self.end() {
            self.end()
        } else {
            time
        }
    }
}
