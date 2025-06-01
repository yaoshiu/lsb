use std::cmp::Ordering;

use log::debug;
use rand::{
    prelude::*,
    seq::index::{IndexVec, sample},
};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

use super::{consts::*, error::*, hash::*, image::*};

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

    let total = build_payload(input, extension, hash)?;

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
    debug!("Data: {} bytes", input.len());

    let image = decode(container)?;

    let capacity_bits = image.len() * lsbs;

    if total_len_bits > capacity_bits {
        return Err(StegError::InsufficientCapacity(format!(
            "Container is too small to hold the data: {} bits required, {} bits available",
            total_len_bits, capacity_bits
        )));
    }

    let image = embed_bytes(image, total, lsbs, seed);

    let output = encode(image, format)?;

    Ok(output)
}

fn embed_bytes(mut image: RgbImage, total: Vec<u8>, lsbs: usize, seed: u64) -> RgbImage {
    let capacity_bits = image.len() * lsbs;

    let order = generate_order(seed, capacity_bits);

    let total_len_bits = total.len() * BITS_PER_BYTE;

    let mut inverse_ord = order
        .iter()
        .take(total_len_bits)
        .enumerate()
        .map(|(i, x)| (x, i))
        .collect::<Vec<_>>();
    inverse_ord.par_sort_by_key(|(x, _)| *x);

    image
        .par_chunks_mut(CHUNK_SIZE)
        .enumerate()
        .for_each(|(index, chunk)| {
            let start = index * CHUNK_SIZE * lsbs;
            let end = start + CHUNK_SIZE * lsbs - 1; // The end should be inclusive so that
            // the upper bound is correct

            let (lower, upper) = bounds(&inverse_ord, start, end);

            for (bit_index, bit_index_seq) in &inverse_ord[lower..upper] {
                let byte_index = bit_index_seq / BITS_PER_BYTE;
                let bit_offset = bit_index_seq % BITS_PER_BYTE;

                let byte = total[byte_index];
                let bit = (byte >> (BITS_PER_BYTE - 1 - bit_offset)) & 1;

                let bit_in_chunk = bit_index / lsbs % CHUNK_SIZE;
                let bit_in_channel = bit_index % lsbs;

                let mask = !(1 << bit_in_channel);
                chunk[bit_in_chunk] = (chunk[bit_in_chunk] & mask) | (bit << bit_in_channel);
            }
        });

    image
}

fn bounds(inverse_ord: &[(usize, usize)], start: usize, end: usize) -> (usize, usize) {
    let lower = inverse_ord
        .binary_search_by(|&(x, _)| match x.cmp(&start) {
            Ordering::Equal => Ordering::Greater,
            ord => ord,
        })
        .unwrap_err();
    let upper = inverse_ord
        .binary_search_by(|&(x, _)| match x.cmp(&end) {
            Ordering::Equal => Ordering::Less,
            ord => ord,
        })
        .unwrap_err();
    (lower, upper)
}

fn generate_order(seed: u64, capacity_bits: usize) -> IndexVec {
    let mut rng = Pcg64Mcg::seed_from_u64(seed);

    // The `amount` parameter must be the same as `total_len_bits` for reproducibility
    sample(&mut rng, capacity_bits, capacity_bits)
}

fn build_payload(input: &[u8], extension: &str, hash: Hash) -> StegResult<Vec<u8>> {
    let ext_len: u8 = extension.len().try_into().map_err(|_| {
        StegError::ExtensionTooLong(format!(
            "Extension length exceeds maximum size: {}",
            extension.len()
        ))
    })?;

    let hash_flag = hash as u8;

    let mut hasher = select_hasher(hash);

    let checksum = use_hasher(&mut *hasher, input);

    let payload = [
        ext_len.to_le_bytes().as_ref(),
        extension.as_bytes(),
        hash_flag.to_le_bytes().as_ref(),
        checksum.as_ref(),
        input,
    ]
    .concat();

    let payload_len: u32 = payload.len().try_into().map_err(|_| {
        StegError::CalculationOverflow(format!(
            "Payload length exceeds maximum size: {} bytes",
            payload.len()
        ))
    })?;

    Ok([payload_len.to_le_bytes().as_ref(), &payload].concat())
}
