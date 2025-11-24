pub mod colormap;
pub mod error;
pub mod image;
pub mod jd79668;
pub mod peripherals;

pub use colormap::*;
pub use error::*;
pub use image::*;
pub use jd79668::*;
pub use peripherals::*;

/// Resolution, of an image or a display, expressed in pixels
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
        }
    }
}
