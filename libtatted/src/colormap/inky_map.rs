//! A colormap specific to the colors supported by the JD79668 display.

use image::{Rgb, imageops::ColorMap};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::InkyError;

#[derive(Copy, Clone, Debug, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum InkyFourColorPalette {
    Black = 0,
    White = 1,
    Yellow = 2,
    Red = 3,
}

// Convert between our colors and image::Rgb values
impl From<InkyFourColorPalette> for Rgb<u8> {
    fn from(color: InkyFourColorPalette) -> Self {
        match color {
            InkyFourColorPalette::Black => Rgb([0, 0, 0]),
            InkyFourColorPalette::White => Rgb([255, 255, 255]),
            InkyFourColorPalette::Yellow => Rgb([255, 255, 0]),
            InkyFourColorPalette::Red => Rgb([255, 0, 0]),
        }
    }
}

// Index into our palette to construct index images
impl TryFrom<usize> for InkyFourColorPalette {
    type Error = InkyError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InkyFourColorPalette::Black),
            1 => Ok(InkyFourColorPalette::White),
            2 => Ok(InkyFourColorPalette::Yellow),
            3 => Ok(InkyFourColorPalette::Red),
            _ => Err(InkyError::OutOfPaletteError),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InkyFourColorMap;

impl ColorMap for InkyFourColorMap {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let mut best_index = 0usize;
        let mut best_distance = i32::MAX;

        for (index, palette_item) in InkyFourColorPalette::iter().enumerate() {
            let palette_color = Rgb::from(palette_item);

            // It would be sweet if image::Rgb<_> implemented ops::Sub, but alas
            let dr = color[0] as i32 - palette_color[0] as i32;
            let dg = color[1] as i32 - palette_color[1] as i32;
            let db = color[2] as i32 - palette_color[2] as i32;
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
        InkyFourColorPalette::try_from(index).map(Rgb::from).ok()
    }

    fn map_color(&self, color: &mut Self::Color) {
        let nearest_color_index = self.index_of(color);
        let nearest_color = self
            .lookup(nearest_color_index)
            .expect("it is a logic error to hit this index out of bounds");

        *color = nearest_color
    }
}
