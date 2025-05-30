//! This crate provides core functionalities for Least Significant Bit (LSB) steganography.
//! It allows embedding data within images and extracting it later.
//!
//! The embedding process involves:
//! 1. Prepending metadata to the input data:
//!    - Length of the original file extension (1 byte).
//!    - Original file extension.
//!    - Hash algorithm flag (1 byte).
//!    - Checksum of the input data.
//! 2. Prepending the total length of this combined payload (4 bytes, little-endian).
//! 3. Encoding this final data into the LSBs of the container image's color channels.
//!    A pseudo-random pixel order is used based on a seed for embedding.
//!
//! The extraction process reverses these steps, using the same seed to read bits
//! in the correct order, verify the checksum, and retrieve the original data and extension.

/// Module for error handling in steganography operations.
pub mod error;
/// Module for hashing functionalities used in steganography.
pub mod hash;

use std::io::Cursor;

use error::{StegError, StegResult};
use hash::Hash;
/// Re-export of `ImageFormat` from the `image` crate.
pub use image::ImageFormat;
use image::ImageReader;
use log::debug;
use rand::{prelude::*, seq::index::sample};
use rand_pcg::Pcg64Mcg;

/// The number of bits in a byte.
const BITS_PER_BYTE: usize = 8;
/// The number of color channels in an image that can be used for embedding (e.g., R, G, B).
const EMBEDDABLE_CHANNELS: usize = 3;
/// A list of image formats considered lossless and suitable for embedding.
const LOSSLESS_FORMATS: [ImageFormat; 10] = [
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

/// Embeds data into a container image using LSB steganography.
///
/// The data is embedded along with metadata: the original file extension,
/// a hash of the data for integrity checking, and the total length of the payload.
///
/// # Arguments
///
/// * `input`: A slice of bytes representing the data to be embedded.
/// * `extension`: The file extension of the input data (e.g., "txt", "jpg").
/// * `container`: A slice of bytes representing the container image data.
/// * `lsbs`: The number of least significant bits to use per color channel for embedding (1-8).
/// * `hash`: The hashing algorithm to use for checksumming the input data.
/// * `seed`: A 64-bit seed for the pseudo-random number generator that determines pixel order.
/// * `format`: The `ImageFormat` of the output image. Must be a lossless format.
///
/// # Returns
///
/// A `StegResult` containing a `Vec<u8>` of the new image data with the embedded content
/// if successful, or a `StegError` if an error occurs.
///
/// # Errors
///
/// This function can return errors for various reasons, including:
/// * `StegError::InvalidLsbValue`: If `lsbs` is not between 1 and `BITS_PER_BYTE`.
/// * `StegError::UnsupportedFormat`: If the specified `format` is not lossless.
/// * `StegError::ExtensionTooLong`: If the `extension` string is too long.
/// * `StegError::InsufficientCapacity`: If the container image is too small to hold the data.
/// * Errors from the `image` crate during image decoding or encoding.
pub fn embed(
    input: &[u8],
    extension: &str,
    container: &[u8],
    lsbs: usize,
    hash: Hash,
    seed: u64,
    format: ImageFormat,
) -> StegResult<Vec<u8>> {
    if lsbs == 0 || lsbs > BITS_PER_BYTE {
        return Err(StegError::InvalidLsbValue(format!(
            "lsbs must be between 1 and {} inclusive",
            BITS_PER_BYTE
        )));
    }

    if !LOSSLESS_FORMATS.contains(&format) {
        return Err(StegError::UnsupportedFormat(format!(
            "Format {:?} is not supported for embedding",
            format
        )));
    }

    let ext_len = extension.len();
    if ext_len > u8::MAX as usize {
        return Err(StegError::ExtensionTooLong(format!(
            "Extension length exceeds maximum size: {}",
            ext_len
        )));
    }
    let ext_len = ext_len as u8;

    let mut hasher = hash::select_hasher(hash);

    let hash_flag = hash as u8;

    let checksum = hash::use_hasher(&mut *hasher, input);

    let payload = [
        ext_len.to_le_bytes().as_ref(),
        extension.as_bytes(),
        hash_flag.to_le_bytes().as_ref(),
        checksum.as_ref(),
        input,
    ]
    .concat();

    let payload_len = payload.len() as u32;

    let total = [payload_len.to_le_bytes().as_ref(), &payload].concat();

    let total_len = total.len();
    // Potential overflow when calculating total_len_bits
    let total_len_bits = total_len.checked_mul(BITS_PER_BYTE).ok_or_else(|| {
        StegError::CalculationOverflow(format!(
            "Overflow calculating total_len_bits: total_len ({}) * BITS_PER_BYTE ({})",
            total_len, BITS_PER_BYTE
        ))
    })?;

    debug!(
        "Preparing to embed: {} bytes ({} bits)",
        total_len, total_len_bits
    );
    debug!("Extension: {} ({} bytes)", extension, ext_len);
    debug!("Data: {} bytes", input.len());

    let container_reader = ImageReader::new(Cursor::new(container)).with_guessed_format()?;

    let mut container = container_reader.decode()?.to_rgb8();

    let width = container.width() as usize;
    let height = container.height() as usize;

    // Potential overflow when calculating width_bits
    let width_bits = width.checked_mul(EMBEDDABLE_CHANNELS)
        .and_then(|res| res.checked_mul(lsbs))
        .ok_or_else(|| StegError::CalculationOverflow(format!(
            "Overflow calculating width_bits: width ({}) * EMBEDDABLE_CHANNELS ({}) * lsbs ({})",
            width, EMBEDDABLE_CHANNELS, lsbs
        )))?;

    // Potential overflow when calculating capacity_bits
    let capacity_bits = width_bits.checked_mul(height).ok_or_else(|| {
        StegError::CalculationOverflow(format!(
            "Overflow calculating capacity_bits: width_bits ({}) * height ({})",
            width_bits, height
        ))
    })?;

    let capacity_bytes = capacity_bits / BITS_PER_BYTE;

    debug!(
        "Container: {} x {} ({} bytes)",
        width, height, capacity_bytes
    );
    debug!("Using {} bits per channel", lsbs);

    if total_len_bits > capacity_bits {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bits required, {} bits available",
            total_len_bits, capacity_bits
        )));
    }

    let mut rng = Pcg64Mcg::seed_from_u64(seed);
    // The `amount` parameter must be the same as `total_len_bits` for reproducibility
    let order = sample(&mut rng, capacity_bits, capacity_bits);

    for (byte_index, &byte) in total.iter().enumerate() {
        for bit_offset in 0..BITS_PER_BYTE {
            let bit = (byte >> (BITS_PER_BYTE - 1 - bit_offset)) & 1;

            // Potential overflow when calculating bit_index
            let bit_index_sequential = byte_index.checked_mul(BITS_PER_BYTE)
                .and_then(|res| res.checked_add(bit_offset))
                .ok_or_else(|| StegError::CalculationOverflow(format!(
                    "Overflow calculating sequential bit_index: byte_index ({}) * BITS_PER_BYTE ({}) + bit_offset ({})",
                    byte_index, BITS_PER_BYTE, bit_offset
                )))?;

            let bit_index = order.index(bit_index_sequential);

            let y = bit_index / width_bits;

            let x_bit = bit_index % width_bits;
            let x = x_bit / (EMBEDDABLE_CHANNELS * lsbs);

            let bit_in_pixel = x_bit % (EMBEDDABLE_CHANNELS * lsbs);

            let channel = bit_in_pixel / lsbs;
            let bit_in_channel = bit_in_pixel % lsbs;

            let pixel = container.get_pixel_mut(x as u32, y as u32);

            let mask = !(1 << bit_in_channel);
            pixel[channel] = (pixel[channel] & mask) | (bit << bit_in_channel);
        }
    }

    let mut output = Vec::new();

    let mut cursor = Cursor::new(&mut output);

    container.write_to(&mut cursor, format)?;

    Ok(output)
}

/// Extracts data embedded in an image using LSB steganography.
///
/// This function attempts to read the payload length, then the payload itself,
/// which includes the original file extension, hash flag, checksum, and the hidden data.
/// It verifies the checksum before returning the data.
///
/// # Arguments
///
/// * `input`: A slice of bytes representing the image data from which to extract content.
/// * `lsbs`: The number of least significant bits per color channel used during embedding (1-8).
/// * `seed`: The 64-bit seed used for the pseudo-random number generator during embedding.
///
/// # Returns
///
/// A `StegResult` containing a tuple `(Vec<u8>, String)` where the `Vec<u8>` is the
/// extracted data and the `String` is the original file extension, if successful.
/// Returns a `StegError` if an error occurs.
///
/// # Errors
///
/// This function can return errors for various reasons, including:
/// * `StegError::InsufficientCapacity`: If the image is too small to contain valid metadata or payload.
/// * `StegError::HashFlagParse`: If the hash flag read from the image is invalid.
/// * `StegError::ChecksumMismatch`: If the checksum of the extracted data does not match the embedded checksum.
/// * Errors from the `image` crate during image decoding.
/// * `std::string::FromUtf8Error` if the extracted extension bytes are not valid UTF-8.
pub fn extract(input: &[u8], lsbs: usize, seed: u64) -> StegResult<(Vec<u8>, String)> {
    let reader = ImageReader::new(Cursor::new(input)).with_guessed_format()?;

    let container = reader.decode()?.to_rgb8();

    let width = container.width() as usize;
    let height = container.height() as usize;

    // Potential overflow when calculating width_bits
    let width_bits = width.checked_mul(EMBEDDABLE_CHANNELS)
        .and_then(|res| res.checked_mul(lsbs))
        .ok_or_else(|| StegError::CalculationOverflow(format!(
            "Overflow calculating width_bits: width ({}) * EMBEDDABLE_CHANNELS ({}) * lsbs ({})",
            width, EMBEDDABLE_CHANNELS, lsbs
        )))?;

    // Potential overflow when calculating capacity_bits
    let capacity_bits = width_bits.checked_mul(height).ok_or_else(|| {
        StegError::CalculationOverflow(format!(
            "Overflow calculating capacity_bits: width_bits ({}) * height ({})",
            width_bits, height
        ))
    })?;
    let capacity_bytes = capacity_bits / BITS_PER_BYTE;

    debug!(
        "Container: {} x {} ({} bytes)",
        width, height, capacity_bytes
    );
    debug!("Using {} bits per channel", lsbs);

    let length_size = core::mem::size_of::<u32>();

    if capacity_bytes < length_size {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bytes available",
            capacity_bytes
        )));
    }

    let length = read_bytes(&container, length_size, lsbs, seed)?;
    let length = u32::from_le_bytes(length.try_into().unwrap()) as usize;
    debug!("Length: {} bytes", length);

    if length + length_size > capacity_bytes {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bytes required, {} bytes available",
            length + length_size,
            capacity_bytes
        )));
    }

    let payload = read_bytes(&container, length + length_size, lsbs, seed)?;
    let payload = &payload[length_size..];

    let ext_len = payload[0] as usize;
    let payload = &payload[1..];
    let extension = String::from_utf8(payload[0..ext_len].to_vec())?;
    debug!("Extension: {} ({} bytes)", extension, ext_len);
    let payload = &payload[ext_len..];

    let hash_flag = payload[0];
    let hash = Hash::from_repr(hash_flag).ok_or(StegError::HashFlagParse(format!(
        "Failed to parse hash: {}",
        hash_flag
    )))?;
    let payload = &payload[1..];
    debug!("Hash: {:?}", hash);

    let mut hasher = hash::select_hasher(hash);
    let hash_length = hasher.output_size();
    let hash_val = &payload[..hash_length];
    let payload = &payload[hash_length..];

    let checksum = hash::use_hasher(&mut *hasher, payload);
    if *checksum != *hash_val {
        return Err(StegError::ChecksumMismatch);
    }

    Ok((payload.to_vec(), extension))
}

/// Reads a specified number of bytes from an image container using LSB steganography.
///
/// This function decodes bits from the LSBs of the image's color channels in a
/// pseudo-random order determined by the seed.
///
/// # Arguments
///
/// * `container`: A reference to an `image::RgbImage` to read from.
/// * `length`: The number of bytes to read.
/// * `lsbs`: The number of least significant bits per color channel used during embedding.
/// * `seed`: The 64-bit seed used for the pseudo-random number generator.
///
/// # Returns
///
/// A `StegResult` containing a `Vec<u8>` of the read bytes if successful,
/// or a `StegError` if an error occurs (e.g., insufficient capacity).
///
/// # Errors
///
/// * `StegError::InsufficientCapacity`: If the image does not have enough capacity
///   to provide the requested number of bytes.
fn read_bytes(
    container: &image::RgbImage,
    length: usize,
    lsbs: usize,
    seed: u64,
) -> StegResult<Vec<u8>> {
    let width = container.width() as usize;
    let height = container.height() as usize;

    // Potential overflow when calculating width_bits
    let width_bits = width.checked_mul(EMBEDDABLE_CHANNELS)
        .and_then(|res| res.checked_mul(lsbs))
        .ok_or_else(|| StegError::CalculationOverflow(format!(
            "Overflow calculating width_bits: width ({}) * EMBEDDABLE_CHANNELS ({}) * lsbs ({})",
            width, EMBEDDABLE_CHANNELS, lsbs
        )))?;

    // Potential overflow when calculating capacity_bits
    let capacity_bits = width_bits.checked_mul(height).ok_or_else(|| {
        StegError::CalculationOverflow(format!(
            "Overflow calculating capacity_bits: width_bits ({}) * height ({})",
            width_bits, height
        ))
    })?;

    // Potential overflow when calculating length_bits
    let length_bits = length.checked_mul(BITS_PER_BYTE).ok_or_else(|| {
        StegError::CalculationOverflow(format!(
            "Overflow calculating length_bits: length ({}) * BITS_PER_BYTE ({})",
            length, BITS_PER_BYTE
        ))
    })?;

    if length_bits > capacity_bits {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bits required, {} bits available",
            length_bits, capacity_bits
        )));
    }

    let mut rng = Pcg64Mcg::seed_from_u64(seed);
    // The `amount` parameter must be the same as `length` fro reproducibility
    let order = sample(&mut rng, capacity_bits, capacity_bits);

    let mut output = vec![0; length];

    for (byte_index, byte) in output.iter_mut().enumerate() {
        for bit_offset in 0..BITS_PER_BYTE {
            // Potential overflow when calculating bit_index
            let bit_index_sequential = byte_index.checked_mul(BITS_PER_BYTE)
                .and_then(|res| res.checked_add(bit_offset))
                .ok_or_else(|| StegError::CalculationOverflow(format!(
                    "Overflow calculating sequential bit_index: byte_index ({}) * BITS_PER_BYTE ({}) + bit_offset ({})",
                    byte_index, BITS_PER_BYTE, bit_offset
                )))?;

            let bit_index = order.index(bit_index_sequential);

            let y = bit_index / width_bits;

            let x_bit = bit_index % width_bits;
            let x = x_bit / (EMBEDDABLE_CHANNELS * lsbs);

            let bit_in_pixel = x_bit % (EMBEDDABLE_CHANNELS * lsbs);

            let channel = bit_in_pixel / lsbs;
            let bit_in_channel = bit_in_pixel % lsbs;

            let pixel = container.get_pixel(x as u32, y as u32);

            let bit = (pixel[channel] >> bit_in_channel) & 1;
            *byte = (*byte << 1) | bit;
        }
    }

    Ok(output)
}
