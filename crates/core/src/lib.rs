//! `iris_core`
//! Foundational data structures, exact rational time math, color spaces,
//! geometry, and universal error definitions for Iris Studio.

pub mod color;
pub mod error;
pub mod geometry;
pub mod pixel;
pub mod time;

pub use color::ColorSpace;
pub use error::{IrisError, IrisResult};
pub use geometry::{Dimensions, Point, Rect};
pub use pixel::PixelFormat;
pub use time::{RationalTime, TimeSpan};
