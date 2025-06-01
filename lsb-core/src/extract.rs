use log::debug;
use rand::{prelude::*, seq::index::sample};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

use super::{consts::*, error::*, hash::*, image::*};

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
    let image = decode(input)?;

    let length = extract_length(&image, lsbs, seed)?;

    extract_payload(&image, length, lsbs, seed)
}

fn extract_payload(
    image: &RgbImage,
    length: usize,
    lsbs: usize,
    seed: u64,
) -> Result<(Vec<u8>, String), StegError> {
    let length_size = core::mem::size_of::<u32>();

    let payload = read_bytes(image, length + length_size, lsbs, seed)?;
    let payload = &payload[length_size..];

    let ext_len = payload[0] as usize;
    let payload = &payload[1..];
    let extension = String::from_utf8(payload[0..ext_len].into())?;
    debug!("Extension: {} ({} bytes)", extension, ext_len);
    let payload = &payload[ext_len..];

    let hash_flag = payload[0];
    let hash = Hash::from_repr(hash_flag).ok_or(StegError::HashFlagParse(format!(
        "Failed to parse hash: {}",
        hash_flag
    )))?;
    let payload = &payload[1..];
    debug!("Hash: {:?}", hash);

    let mut hasher = select_hasher(hash);
    let hash_length = hasher.output_size();
    let hash_val = &payload[..hash_length];
    let payload = &payload[hash_length..];

    let checksum = use_hasher(&mut *hasher, payload);
    if *checksum != *hash_val {
        return Err(StegError::ChecksumMismatch);
    }

    Ok((payload.to_vec(), extension))
}

fn extract_length(image: &RgbImage, lsbs: usize, seed: u64) -> StegResult<usize> {
    let capacity_bytes = image.len();

    let length_size = core::mem::size_of::<u32>();
    if capacity_bytes < length_size {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bytes available",
            capacity_bytes
        )));
    }
    let length = read_bytes(image, length_size, lsbs, seed)?;
    let length = u32::from_le_bytes(length.try_into().unwrap()) as usize;
    debug!("Length: {} bytes", length);
    if length + length_size > capacity_bytes {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bytes required, {} bytes available",
            length + length_size,
            capacity_bytes
        )));
    }

    Ok(length)
}

fn read_bytes(
    container: &image::RgbImage,
    length: usize,
    lsbs: usize,
    seed: u64,
) -> StegResult<Vec<u8>> {
    let width = container.width() as usize;
    let height = container.height() as usize;

    // Potential overflow when calculating width_bits
    let width_bits = width
        .checked_mul(EMBEDDABLE_CHANNELS)
        .and_then(|res| res.checked_mul(lsbs))
        .ok_or_else(|| {
            StegError::CalculationOverflow(format!(
            "Overflow calculating width_bits: width ({}) * EMBEDDABLE_CHANNELS ({}) * lsbs ({})",
            width, EMBEDDABLE_CHANNELS, lsbs
        ))
        })?;

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

    output.par_chunks_mut(CHUNK_SIZE).enumerate().try_for_each(|(index, chunk)| -> StegResult<()> {
        for (byte_index, byte) in chunk.iter_mut().enumerate() {
            let byte_index = index * CHUNK_SIZE + byte_index;

            for bit_offset in 0..BITS_PER_BYTE {
                // Potential overflow when calculating bit_index
                let bit_index_seq = byte_index.checked_mul(BITS_PER_BYTE)
                    .and_then(|res| res.checked_add(bit_offset))
                    .ok_or_else(|| StegError::CalculationOverflow(format!(
                        "Overflow calculating sequential bit_index: byte_index ({}) * BITS_PER_BYTE ({}) + bit_offset ({})",
                        byte_index, BITS_PER_BYTE, bit_offset
                    )))?;

                let bit_index = order.index(bit_index_seq);

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

        Ok(())
    })?;

    Ok(output)
}
