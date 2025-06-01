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

/// Module for constants used throughout the crate.
mod consts;
/// Module for embedding data into images using LSB steganography.
mod embed;
/// Module for error handling in steganography operations.
pub mod error;
/// Module for extracting data from images using LSB steganography.
mod extract;
/// Module for hashing functionalities used in steganography.
pub mod hash;
/// Module for image handling, including decoding and encoding images.
pub mod image;

pub use embed::embed;
pub use extract::extract;
