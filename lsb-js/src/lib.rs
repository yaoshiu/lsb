mod error;
pub mod utils;

use std::str::FromStr;

use error::LsbError;
use log::Level;
use lsb_core::ImageFormat;
use wasm_bindgen::prelude::*;

/// Exposes the JavaScript `alert` function.
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

/// Embeds a payload into a container image.
///
/// # Arguments
///
/// * `input` - The payload data to embed.
/// * `extension` - The file extension of the payload.
/// * `container` - The container image data.
/// * `lsbs` - The number of least significant bits to use for encoding. Defaults to 1.
/// * `hash` - The hashing algorithm to use. Defaults to "BLAKE3".
/// * `seed` - The seed for the random number generator. Defaults to 42.
/// * `format` - The image format of the container. Defaults to "PNG".
///
/// # Returns
///
/// A `Result` containing the new image data with the embedded payload, or an `LsbError` if an error occurs.
#[wasm_bindgen]
pub fn embed(
    input: &[u8],
    extension: &str,
    container: &[u8],
    lsbs: Option<usize>,
    hash: Option<String>,
    seed: Option<u64>,
    format: Option<String>,
) -> Result<Vec<u8>, LsbError> {
    let lsbs = lsbs.unwrap_or(1);
    let hash = hash.unwrap_or("BLAKE3".to_string());
    let seed = seed.unwrap_or(42);
    let format = format.unwrap_or("PNG".to_string());

    let hash = lsb_core::hash::Hash::from_str(&hash)?;

    let format = ImageFormat::from_extension(&format).ok_or(LsbError::Steg(
        lsb_core::error::StegError::UnsupportedFormat(format!(
            "Unsupported image format: {}",
            format
        )),
    ))?;

    Ok(lsb_core::embed(
        input, extension, container, lsbs, hash, seed, format,
    )?)
}

/// Represents the result of an extraction operation.
#[wasm_bindgen]
pub struct ExtractResult(
    /// The extracted payload data.
    #[wasm_bindgen(getter_with_clone)]
    pub Vec<u8>,
    /// The file extension of the extracted payload.
    #[wasm_bindgen(getter_with_clone)]
    pub String,
);

/// Extracts a payload from a container image.
///
/// # Arguments
///
/// * `container` - The container image data.
/// * `lsbs` - The number of least significant bits used for encoding. Defaults to 1.
/// * `seed` - The seed for the random number generator. Defaults to 42.
///
/// # Returns
///
/// A `Result` containing an `ExtractResult` with the extracted payload and its extension,
/// or an `LsbError` if an error occurs.
#[wasm_bindgen]
pub fn extract(
    container: &[u8],
    lsbs: Option<usize>,
    seed: Option<u64>,
) -> Result<ExtractResult, LsbError> {
    let lsbs = lsbs.unwrap_or(1);
    let seed = seed.unwrap_or(42);

    let (data, extension) = lsb_core::extract(container, lsbs, seed)?;

    Ok(ExtractResult(data, extension))
}

/// Initializes the logger with a specified log level.
///
/// # Arguments
///
/// * `level` - The log level to set. Defaults to "info".
///
/// # Returns
///
/// A `Result` indicating success or an `LsbError` if an error occurs.
#[wasm_bindgen]
pub fn init_logger(level: Option<String>) -> Result<(), LsbError> {
    let level = level.unwrap_or("info".to_string());
    let level = Level::from_str(&level)?;

    console_log::init_with_level(level)?;

    Ok(())
}

/// A simple greeting function that calls the JavaScript `alert` function.
#[wasm_bindgen]
pub fn greet() {
    alert("Hello, lsb-js!");
}

/// The main entry point for the Wasm module.
///
/// Initializes the panic hook and logger.
#[wasm_bindgen(main)]
fn main() -> Result<(), LsbError> {
    utils::set_panic_hook();
    init_logger(None)?;

    Ok(())
}
