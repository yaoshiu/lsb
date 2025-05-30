use std::fmt;

/// Represents the possible errors that can occur during steganography operations.
#[derive(Debug)]
pub enum StegError {
    /// Error indicating an invalid LSB (Least Significant Bit) value was encountered.
    InvalidLsbValue(String),
    /// Error originating from the underlying image processing library.
    ImageProcessing(image::ImageError),
    /// Error during the detection of the image format.
    FormatDetection(String),
    /// Error indicating that the file extension is too long to be embedded.
    ExtensionTooLong(String),
    /// Error indicating that the container image does not have enough capacity to hold the payload.
    InsufficientCapacity(String),
    /// Error occurring during the parsing of the payload data.
    PayloadParse(String),
    /// Error indicating a mismatch in checksums, suggesting data corruption.
    ChecksumMismatch,
    /// Error due to a numeric calculation overflow.
    CalculationOverflow(String),
    /// Error indicating that the calculated capacity exceeds the maximum value of `usize`.
    CapacityExceedsUsizeMax(String),
    /// Error occurring during the parsing of a hash flag.
    HashFlagParse(String),
    /// Error indicating that the image format is not supported.
    UnsupportedFormat(String),
    /// General I/O error.
    Io(std::io::Error),
}

impl fmt::Display for StegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StegError::InvalidLsbValue(msg) => write!(f, "Invalid LSBs value: {}", msg),
            StegError::ImageProcessing(err) => write!(f, "Image processing error: {}", err),
            StegError::FormatDetection(msg) => write!(f, "Image format detection error: {}", msg),
            StegError::ExtensionTooLong(msg) => write!(f, "Extension too long: {}", msg),
            StegError::InsufficientCapacity(msg) => {
                write!(f, "Insufficient container capacity: {}", msg)
            }
            StegError::PayloadParse(msg) => write!(f, "Failed to parse payload: {}", msg),
            StegError::ChecksumMismatch => write!(f, "Checksum mismatch"),
            StegError::CalculationOverflow(msg) => {
                write!(f, "Numeric calculation overflow: {}", msg)
            }
            StegError::CapacityExceedsUsizeMax(msg) => {
                write!(f, "Capacity exceeds system limit (usize::MAX): {}", msg)
            }
            StegError::Io(err) => write!(f, "I/O error: {}", err),
            StegError::HashFlagParse(msg) => write!(f, "Failed to parse hash flag: {}", msg),
            StegError::UnsupportedFormat(msg) => write!(f, "Unsupported image format: {}", msg),
        }
    }
}

impl std::error::Error for StegError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StegError::ImageProcessing(err) => Some(err),
            StegError::Io(err) => Some(err),
            _ => None,
        }
    }
}

/// Converts an `image::ImageError` into a `StegError::ImageProcessing` variant.
impl From<image::ImageError> for StegError {
    fn from(err: image::ImageError) -> Self {
        StegError::ImageProcessing(err)
    }
}

/// Converts a `std::io::Error` into a `StegError::Io` variant.
impl From<std::io::Error> for StegError {
    fn from(err: std::io::Error) -> Self {
        StegError::Io(err)
    }
}

/// Converts a `std::string::FromUtf8Error` into a `StegError::PayloadParse` variant.
impl From<std::string::FromUtf8Error> for StegError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        StegError::PayloadParse(format!("Invalid UTF-8 sequence in extension: {}", err))
    }
}

/// A type alias for `Result<T, StegError>`, used for functions that can return a `StegError`.
pub type StegResult<T> = Result<T, StegError>;
