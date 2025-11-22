use strum::Display;

/// Colors supported by the JD79668
#[derive(Copy, Clone, Debug, Display)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedColors {
    Black,
    White,
    Yellow,
    Red,
}
