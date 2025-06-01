mod error;

use std::{borrow::Cow, str::FromStr};

use error::LsbError;
use lsb_core::{error::StegError, hash, image::ImageFormat};
use pyo3::prelude::*;

/// Embeds a payload into a container image.
///
/// Args:
///     input (bytes): The payload to embed.
///     extension (str): The extension of the payload.
///     container (bytes): The container image.
///     lsbs (int): The number of least significant bits to use.
///     hash (str): The hash algorithm to use.
///     seed (int): The seed for the random number generator.
///     format (str): The format of the container image.
///
/// Returns:
///     bytes: The container image with the embedded payload.
///
/// Raises:
///     LsbError: If an error occurs during embedding.
#[pyfunction]
#[pyo3(
    signature = (input, extension, container, lsbs=1, hash="BLAKE3", seed=42, format="PNG")
)]
fn embed<'a>(
    input: &[u8],
    extension: &str,
    container: &[u8],
    lsbs: usize,
    hash: &str,
    seed: u64,
    format: &str,
) -> Result<Cow<'a, [u8]>, LsbError> {
    let hash = hash::Hash::from_str(hash)?;

    let format = ImageFormat::from_extension(format).ok_or(LsbError::Steg(
        StegError::UnsupportedFormat(format!("Unsupported image format: {}", format)),
    ))?;

    Ok(lsb_core::embed(input, extension, container, lsbs, hash, seed, format)?.into())
}

/// Extracts a payload from a container image.
///
/// Args:
///     input (bytes): The container image with the embedded payload.
///     lsbs (int): The number of least significant bits used for embedding.
///     seed (int): The seed for the random number generator used for embedding.
///
/// Returns:
///     tuple[bytes, str]: A tuple containing the extracted payload and its extension.
///
/// Raises:
///     LsbError: If an error occurs during extraction.
#[pyfunction]
#[pyo3(signature = (input, lsbs=1, seed=42))]
fn extract<'a>(input: &[u8], lsbs: usize, seed: u64) -> Result<(Cow<'a, [u8]>, String), LsbError> {
    let (data, ext) = lsb_core::extract(input, lsbs, seed)?;

    Ok((data.into(), ext))
}

/// A Python module implementing LSB steganography.
#[pymodule]
fn lsb_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_function(wrap_pyfunction!(embed, m)?)?;
    m.add_function(wrap_pyfunction!(extract, m)?)?;

    Ok(())
}
