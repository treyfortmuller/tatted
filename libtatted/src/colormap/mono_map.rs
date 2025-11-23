//! A colormap consisting of only black and white. Similar to the [`image::imageops::BiLevel`] colormap
//! but operates on [`image::Rgb`] rather than [`image::Luma`] to eliminate some complicated generics.

use image::{Rgb, imageops::ColorMap};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::InkyError;

#[derive(Copy, Clone, Debug, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum MonoColorPalette {
    Black = 0,
    White = 1,
}

// Convert between our colors and image::Rgb values
impl From<MonoColorPalette> for Rgb<u8> {
    fn from(color: MonoColorPalette) -> Self {
        match color {
            MonoColorPalette::Black => Rgb([0, 0, 0]),
            MonoColorPalette::White => Rgb([255, 255, 255]),
        }
    }
}

// Index into our palette to construct index images
impl TryFrom<usize> for MonoColorPalette {
    type Error = InkyError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MonoColorPalette::Black),
            1 => Ok(MonoColorPalette::White),
            _ => Err(InkyError::OutOfPaletteError),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MonoColorMap;

impl ColorMap for MonoColorMap {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let mut best_index = 0usize;
        let mut best_distance = i32::MAX;

        for (index, palette_item) in MonoColorPalette::iter().enumerate() {
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
        MonoColorPalette::try_from(index)
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
