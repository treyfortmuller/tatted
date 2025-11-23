use crate::{InkyResult, Resolution};
use camino::Utf8PathBuf;
use image::imageops::colorops::{ColorMap, index_colors};
use image::{DynamicImage, ImageBuffer, ImageReader, Luma, Rgb};

/// An image buffer which contains 1 byte per pixel, with each pixel referring to an index in a color palette,
/// not actually "luminance".
type IndexImage = ImageBuffer<Luma<u8>, Vec<u8>>;

pub struct ImagePreProcessor<CMap: ColorMap<Color = Rgb<u8>>> {
    pub color_map: CMap,
    pub desired_res: Resolution,
}

impl<CMap: ColorMap<Color = Rgb<u8>>> ImagePreProcessor<CMap> {
    pub fn new(color_map: CMap, desired_res: Resolution) -> Self {
        Self {
            color_map,
            desired_res,
        }
    }

    pub fn prepare(&self, img: &DynamicImage, dither: bool) -> InkyResult<InkyImage> {
        let input_res = Resolution::new(img.width(), img.height());

        // In the future we could do some kind of intelligent resizing or something, but for now just
        // throw an error if we don't get the native resolution of the display we're using.
        if input_res != self.desired_res {
            return Err(crate::InkyError::UnsupportedResolution {
                expected: self.desired_res,
                found: input_res,
            });
        }

        let rgb = &mut img.to_rgb8();
        if dither {
            image::imageops::dither(rgb, &self.color_map);
        }
        let index_image = index_colors(&rgb, &self.color_map);

        // Remap to a colorspace we can encode for saving prepared images to the filesystem
        let mapped = image::ImageBuffer::from_fn(
            self.desired_res.width.into(),
            self.desired_res.height.into(),
            |x, y| {
                let p = index_image.get_pixel(x, y);
                self.color_map
                    .lookup(p.0[0] as usize)
                    .expect("indexed color out-of-range")
            },
        );

        Ok(InkyImage::new(index_image, DynamicImage::from(mapped)))
    }

    pub fn prepare_from_path(&self, path: Utf8PathBuf, dither: bool) -> InkyResult<InkyImage> {
        let img = ImageReader::open(path)?.decode()?;
        self.prepare(&img, dither)
    }
}

/// A pre-processed image containing an [`IndexImage`] ready to be bit-packed and serialized for the display,
/// as well as a [`DynamicImage`] containing pixel color information representative of whats contained within
/// the [`IndexImage`], ready to be encoded and saved to the filesystem for viewing.
pub struct InkyImage {
    index_img: IndexImage,
    pixel_img: DynamicImage,
}

impl InkyImage {
    pub fn new(index_img: IndexImage, pixel_img: DynamicImage) -> Self {
        Self {
            index_img,
            pixel_img,
        }
    }

    /// Saves the buffer to a file with the format derived from the file extension.
    pub fn save(&self, path: Utf8PathBuf) -> InkyResult<()> {
        self.pixel_img.save(path)?;
        Ok(())
    }
}
