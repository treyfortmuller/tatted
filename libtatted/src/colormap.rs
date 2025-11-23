use image::{Rgb, imageops::ColorMap};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::InkyError;

/// Colors supported by the JD79668
#[derive(Copy, Clone, Debug, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedColors {
    Black = 0,
    White = 1,
    Yellow = 2,
    Red = 3,
}

impl From<SupportedColors> for Rgb<u8> {
    fn from(color: SupportedColors) -> Self {
        match color {
            SupportedColors::Black => Rgb([0, 0, 0]),
            SupportedColors::White => Rgb([255, 255, 255]),
            SupportedColors::Yellow => Rgb([255, 255, 0]),
            SupportedColors::Red => Rgb([255, 0, 0]),
        }
    }
}

impl TryFrom<usize> for SupportedColors {
    type Error = InkyError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SupportedColors::Black),
            1 => Ok(SupportedColors::White),
            2 => Ok(SupportedColors::Yellow),
            3 => Ok(SupportedColors::Red),
            _ => Err(InkyError::OutOfPaletteError),
        }
    }
}

// TODO (tff): combine with the above enum
#[derive(Copy, Clone)]
pub struct InkyFourColorPalette;

impl ColorMap for InkyFourColorPalette {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let mut best_index = 0usize;
        let mut best_distance = i32::MAX;

        for (index, palette_item) in SupportedColors::iter().enumerate() {
            let palette_color = Rgb::from(palette_item);

            // It would be sweet if image::Rgb<_> implemented ops::Sub, but alas
            let dr = (color[0] as i32 - palette_color[0] as i32);
            let dg = (color[1] as i32 - palette_color[1] as i32);
            let db = (color[2] as i32 - palette_color[2] as i32);
            let distance = dr.pow(2) + dg.pow(2) + db.pow(2);

            if distance < best_distance {
                best_distance = distance;
                best_index = index;
            }
        }

        best_index
    }

    fn has_lookup(&self) -> bool {
        true
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        SupportedColors::try_from(index)
            .map(|color| Rgb::from(color))
            .ok()
    }

    fn map_color(&self, color: &mut Self::Color) {
        let nearest_color_index = self.index_of(color);
        let nearest_color = self
            .lookup(nearest_color_index)
            .expect("it is a logic error to hit this index out of bounds");

        *color = Rgb::from(nearest_color)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO (tff)
    #[test]
    fn basic_mapping() {}
}
