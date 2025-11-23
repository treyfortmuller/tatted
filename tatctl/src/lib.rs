use clap::ValueEnum;
use libtatted::{InkyFourColorMap, InkyFourColorPalette, MonoColorMap, SupportedColorMaps};
use strum::Display;

/// Colors supported by the JD79668, a mirror of [`libtatted::InkyFourColorPalette`]` for use with clap.
#[derive(Copy, Clone, Debug, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
pub enum CliColors {
    Black,
    White,
    Yellow,
    Red,
}

impl From<CliColors> for InkyFourColorPalette {
    fn from(value: CliColors) -> Self {
        match value {
            CliColors::Black => InkyFourColorPalette::Black,
            CliColors::White => InkyFourColorPalette::White,
            CliColors::Yellow => InkyFourColorPalette::Yellow,
            CliColors::Red => InkyFourColorPalette::Red,
        }
    }
}

/// Supported color maps for spatial quantization of images
#[derive(Debug, Copy, Clone, ValueEnum, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum CliColorMaps {
    InkyFourColor,
    Mono,
}

impl From<CliColorMaps> for SupportedColorMaps {
    fn from(value: CliColorMaps) -> Self {
        match value {
            CliColorMaps::InkyFourColor => SupportedColorMaps::InkyFourColor(InkyFourColorMap),
            CliColorMaps::Mono => SupportedColorMaps::Mono(MonoColorMap),
        }
    }
}
