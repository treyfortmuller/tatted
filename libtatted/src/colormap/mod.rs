//! Custom types which implement [`image::imageops::ColorMap`] for spatial quantization (indexing and dithering)
//! of images as part of the preprocessing pipeline for e-ink rendering.

pub mod inky_map;
pub mod mono_map;

pub use inky_map::*;
pub use mono_map::*;

/// Supported color maps for spatial quantization of images
#[derive(Debug, Copy, Clone)]
pub enum SupportedColorMaps {
    InkyFourColor(InkyFourColorMap),
    Mono(MonoColorMap),
}
