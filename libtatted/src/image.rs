use camino::Utf8PathBuf;
use image::imageops::colorops::{ColorMap, index_colors};
use image::imageops::dither;
use image::{DynamicImage, GrayImage, ImageFormat, ImageReader, Luma, RgbImage};

use crate::{InkyFourColorMap, InkyFourColorPalette, InkyResult, Resolution};

// Re-export useful ColorMaps
pub use image::imageops::colorops::BiLevel;

pub struct ImagePreProcessor<CMap: ColorMap> {
    pub color_map: CMap,
    pub desired_res: Resolution,
}

impl<CMap: ColorMap> ImagePreProcessor<CMap> {
    pub fn new(color_map: CMap, desired_res: Resolution) -> Self {
        Self {
            color_map,
            desired_res,
        }
    }
}

impl ImagePreProcessor<BiLevel> {
    pub fn prepare(&self, img: &DynamicImage) -> InkyResult<IndexImage> {
        let input_res = Resolution::new(img.width() as u16, img.height() as u16);

        // In the future we could do some kind of intelligent resizing or something, but for now just
        // throw an error if we don't get the native resolution of the display we're using.
        if input_res != self.desired_res {
            return Err(crate::InkyError::UnsupportedResolution {
                expected: self.desired_res,
                found: input_res,
            });
        }

        let luma = img.to_luma8();
        let index_image = index_colors(&luma, &self.color_map);

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

        Ok(IndexImage::new(
            DynamicImage::from(index_image),
            DynamicImage::from(mapped),
        ))
    }

    pub fn prepare_from_path(&self, path: Utf8PathBuf) -> InkyResult<IndexImage> {
        let img = ImageReader::open(path)?.decode()?;
        self.prepare(&img)
    }
}

// TODO (tff): eliminate this duplication

impl ImagePreProcessor<InkyFourColorMap> {
    pub fn prepare(&self, img: &DynamicImage) -> InkyResult<IndexImage> {
        let input_res = Resolution::new(img.width() as u16, img.height() as u16);

        // In the future we could do some kind of intelligent resizing or something, but for now just
        // throw an error if we don't get the native resolution of the display we're using.
        if input_res != self.desired_res {
            return Err(crate::InkyError::UnsupportedResolution {
                expected: self.desired_res,
                found: input_res,
            });
        }

        let rgb = img.to_rgb8();
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

        Ok(IndexImage::new(
            DynamicImage::from(index_image),
            DynamicImage::from(mapped),
        ))
    }

    pub fn prepare_dither(&self, img: &DynamicImage) -> InkyResult<IndexImage> {
        let input_res = Resolution::new(img.width() as u16, img.height() as u16);

        // In the future we could do some kind of intelligent resizing or something, but for now just
        // throw an error if we don't get the native resolution of the display we're using.
        if input_res != self.desired_res {
            return Err(crate::InkyError::UnsupportedResolution {
                expected: self.desired_res,
                found: input_res,
            });
        }

        let rgb = &mut img.to_rgb8();
        dither(rgb, &self.color_map);
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

        Ok(IndexImage::new(
            DynamicImage::from(index_image),
            DynamicImage::from(mapped),
        ))
    }

    pub fn prepare_from_path(&self, path: Utf8PathBuf) -> InkyResult<IndexImage> {
        let img = ImageReader::open(path)?.decode()?;
        self.prepare(&img)
    }

    pub fn prepare_dither_from_path(&self, path: Utf8PathBuf) -> InkyResult<IndexImage> {
        let img = ImageReader::open(path)?.decode()?;
        self.prepare_dither(&img)
    }
}

pub struct IndexImage {
    index_img: DynamicImage,
    pixel_img: DynamicImage, // TODO: generic over pixel type
}

impl IndexImage {
    pub fn new(index_img: DynamicImage, pixel_img: DynamicImage) -> Self {
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
