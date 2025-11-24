use crate::{InkyResult, Resolution};
use camino::Utf8PathBuf;
use image::Pixel;
use image::imageops::colorops::{ColorMap, index_colors};
use image::{DynamicImage, ImageBuffer, ImageReader, Luma};

/// An image buffer which contains 1 byte per pixel, with each pixel referring to an index in a color palette,
/// not actually "luminance".
type IndexImage = ImageBuffer<Luma<u8>, Vec<u8>>;

// Re-export Rgb, we leak that to the public interface
pub use image::Rgb;

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

    /// Preprocess the argument [`DynamicImage`] performing color quantization and optionally dithering.
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
        let index_image = index_colors(rgb, &self.color_map);

        // Remap to a colorspace we can encode for saving prepared images to the filesystem
        let mapped = self.map_index_image(&index_image)?;

        Ok(InkyImage::new(index_image, mapped))
    }

    /// Given an [`IndexImage`], map back into a "colorspace" image using the color map for this [`ImagePreProcessor`].
    fn map_index_image(&self, index_image: &IndexImage) -> InkyResult<DynamicImage> {
        // Remap to a colorspace we can encode for saving prepared images to the filesystem
        let mapped =
            image::ImageBuffer::from_fn(self.desired_res.width, self.desired_res.height, |x, y| {
                let p = index_image.get_pixel(x, y);
                self.color_map
                    .lookup(p.0[0] as usize)
                    .expect("indexed color out-of-range")
            });

        Ok(DynamicImage::from(mapped))
    }

    /// Preprocess the image file at the argument filepath, performing color quantization and optionally dithering.
    /// Jpegs, PNGs, and BMPs are supported.
    pub fn prepare_from_path(&self, path: Utf8PathBuf, dither: bool) -> InkyResult<InkyImage> {
        let img = ImageReader::open(path)?.decode()?;
        self.prepare(&img, dither)
    }

    /// Generates a solid-color image of the correct dimensions and then quantizes it using the configured
    /// color map. Typically you'd use [`Rgb<u8>`] values which belong to the color palette used
    /// by the display we're generating for.
    pub fn new_color(&self, color: Rgb<u8>) -> InkyResult<InkyImage> {
        let w = self.desired_res.width;
        let h = self.desired_res.height;

        let pix = *Luma::from_slice(&[self.color_map.index_of(&color) as u8]);

        let index_image = IndexImage::from_pixel(w, h, pix);
        let mapped = self.map_index_image(&index_image)?;

        Ok(InkyImage::new(index_image, mapped))
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

    /// Returns a clone of the palletized image
    pub fn index_img(&self) -> IndexImage {
        self.index_img.clone()
    }

    /// Returns a clone of the color-pixel image
    pub fn pixel_img(&self) -> DynamicImage {
        self.pixel_img.clone()
    }

    /// Returns the resolution of the inky image
    pub fn resolution(&self) -> Resolution {
        let w = self.index_img.width();
        let h = self.index_img.height();

        Resolution::new(w, h)
    }
}
