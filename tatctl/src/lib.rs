use clap::ValueEnum;
use libtatted::SupportedColors;
use strum::Display;

/// Colors supported by the JD79668, a mirror of libtatted::SupportedColors for use with clap.
#[derive(Copy, Clone, Debug, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
pub enum CliColors {
    Black,
    White,
    Yellow,
    Red,
}

impl From<CliColors> for SupportedColors {
    fn from(value: CliColors) -> Self {
        match value {
            CliColors::Black => SupportedColors::Black,
            CliColors::White => SupportedColors::White,
            CliColors::Yellow => SupportedColors::Yellow,
            CliColors::Red => SupportedColors::Red,
        }
    }
}
