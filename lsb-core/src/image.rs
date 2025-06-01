use std::io::Cursor;

use super::error::StegResult;
use image::ImageReader;
pub use image::{ImageFormat, RgbImage};

/// A list of image formats considered lossless and suitable for embedding.
pub const LOSSLESS_FORMATS: [ImageFormat; 10] = [
    ImageFormat::Png,
    ImageFormat::WebP,
    ImageFormat::Pnm,
    ImageFormat::Tiff,
    ImageFormat::Tga,
    ImageFormat::Bmp,
    ImageFormat::Ico,
    ImageFormat::Hdr,
    ImageFormat::Farbfeld,
    ImageFormat::Qoi,
];

pub(crate) fn decode(container: &[u8]) -> StegResult<RgbImage> {
    let container_reader = ImageReader::new(Cursor::new(container)).with_guessed_format()?;
    let image = container_reader.decode()?.to_rgb8();
    Ok(image)
}

pub(crate) fn encode(image: RgbImage, format: ImageFormat) -> StegResult<Vec<u8>> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    image.write_to(&mut cursor, format)?;
    Ok(output)
}
