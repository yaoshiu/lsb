use core::fmt;

use lsb_core::{error::StegError, hash::ParseError};
use pyo3::prelude::*;

#[derive(Debug)]
pub enum LsbError {
    Steg(StegError),
    Parse(ParseError),
}

impl fmt::Display for LsbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LsbError::Steg(err) => write!(f, "{}", err),
            LsbError::Parse(err) => write!(f, "ParseError: {}", err),
        }
    }
}

impl std::error::Error for LsbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LsbError::Steg(err) => Some(err),
            LsbError::Parse(err) => Some(err),
        }
    }
}

impl From<ParseError> for LsbError {
    fn from(err: ParseError) -> Self {
        LsbError::Parse(err)
    }
}

impl From<StegError> for LsbError {
    fn from(err: StegError) -> Self {
        LsbError::Steg(err)
    }
}

impl std::convert::From<LsbError> for PyErr {
    fn from(err: LsbError) -> Self {
        match err {
            LsbError::Steg(steg_err) => {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(steg_err.to_string())
            }
            LsbError::Parse(parse_err) => {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(parse_err.to_string())
            }
        }
    }
}
