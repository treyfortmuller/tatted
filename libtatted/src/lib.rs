pub mod colormap;
pub mod error;
pub mod image;
pub mod jd79668;

pub use colormap::*;
pub use error::*;
pub use image::*;
pub use jd79668::*;

/// Resolution, of an image or a display, expressed in pixels
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub width: u16,
    pub height: u16,
}

impl Resolution {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            width: w,
            height: h,
        }
    }
}
