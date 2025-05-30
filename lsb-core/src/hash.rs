use clap::ValueEnum;
use digest::DynDigest;
pub use strum::ParseError;
use strum::{EnumString, FromRepr};

/// Represents the available hashing algorithms.
///
/// This enum is used to specify which hashing algorithm to use for various operations.
/// It derives several traits for convenience, including:
/// - `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`: Standard Rust traits.
/// - `ValueEnum`: For integration with `clap` command-line argument parsing.
/// - `EnumString`: To allow parsing from string representations (e.g., "SHA256").
/// - `FromRepr`: To allow conversion from its underlying integer representation.
///
/// The `strum(serialize_all = "UPPERCASE")` attribute ensures that string representations
/// (e.g., for parsing or display) use uppercase names like "BLAKE3", "SHA256", etc.
/// The `repr(u8)` attribute specifies that the enum is represented by an 8-bit unsigned integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, EnumString, FromRepr)]
#[strum(serialize_all = "UPPERCASE")]
#[repr(u8)]
pub enum Hash {
    Blake3 = 0,
    Sha256 = 1,
    Sha512 = 2,
    Sha1 = 3,
}

/// Updates the given hasher with data and returns the resulting hash.
///
/// This function takes a mutable reference to a dynamic digest object (`DynDigest`),
/// updates it with the provided `data` slice, and then finalizes the hash computation.
/// The hasher is reset after finalization, making it ready for reuse.
///
/// # Arguments
///
/// * `hasher`: A mutable reference to a trait object implementing `DynDigest`.
///   This is the hashing algorithm instance to use.
/// * `data`: A byte slice (`&[u8]`) containing the data to be hashed.
///
/// # Returns
///
/// A `Box<[u8]>` containing the computed hash digest.
pub fn use_hasher(hasher: &mut dyn DynDigest, data: &[u8]) -> Box<[u8]> {
    hasher.update(data);
    hasher.finalize_reset()
}

/// Selects and returns a boxed hasher instance based on the `Hash` enum variant.
///
/// This function acts as a factory for creating instances of different hashing algorithms
/// supported by the `Hash` enum.
///
/// # Arguments
///
/// * `hash`: A `Hash` enum variant specifying the desired hashing algorithm.
///
/// # Returns
///
/// A `Box<dyn DynDigest>` which is a trait object pointing to an instance of the
/// selected hashing algorithm. This allows for dynamic dispatch of hash operations.
///
/// # Examples
///
/// ```
/// use lsb_core::hash::{Hash, select_hasher}; // Assuming lsb_core is the crate name
///
/// let sha256_hasher = select_hasher(Hash::Sha256);
/// let blake3_hasher = select_hasher(Hash::Blake3);
/// // Now sha256_hasher and blake3_hasher can be used with functions like use_hasher
/// ```
pub fn select_hasher(hash: Hash) -> Box<dyn DynDigest> {
    use Hash::*;

    match hash {
        Blake3 => Box::new(blake3::Hasher::new()),
        Sha256 => Box::new(sha2::Sha256::default()),
        Sha512 => Box::new(sha2::Sha512::default()),
        Sha1 => Box::new(sha1::Sha1::default()),
    }
}
