use clap::ValueEnum;
use libtatted::InkyFourColorPalette;
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
