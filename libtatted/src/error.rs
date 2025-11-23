use std::time::Duration;
use thiserror::Error;

use crate::Resolution;

/// Error variants for interacting with Inky displays.
#[derive(Debug, Error)]
pub enum InkyError {
    #[error("waiting on the busy pin timed out after {}ms", timeout.as_millis())]
    BusyTimeout { timeout: Duration },

    // TODO (tff): might want more context here on the pin number and a name
    #[error("GPIO error: {0}")]
    GpioError(#[from] gpiocdev::Error),

    #[error("SPI IO error: {0}")]
    SpiIoError(#[from] std::io::Error),

    #[error("the display is uninitialized")]
    Uninitialized,

    #[error(
        "image buffer is palletized incorrectly, pixel values must be in [{}, {}]",
        index_min,
        index_max
    )]
    InvalidPalettization {
        /// The minimum index expected for this display's color palette
        index_min: usize,

        /// The maximum index expected for this display's color palette
        index_max: usize,
    },

    #[error(
        "image buffer is the wrong length, expected {} and found {}",
        expected,
        found
    )]
    InvalidBufferLength {
        /// The expected size of a palletized image buffer
        expected: usize,

        /// The buffer length discovered for bit-packing
        found: usize,
    },

    #[error("error during image encoding or decoding: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("input image is an unsupported resolution, expected {}x{} found {}x{}", expected.width, expected.height, found.width, found.height)]
    UnsupportedResolution {
        expected: Resolution,
        found: Resolution,
    },

    #[error("color to be rendered was outside of the supported color palette")]
    OutOfPaletteError,
}

pub type InkyResult<T> = Result<T, InkyError>;
