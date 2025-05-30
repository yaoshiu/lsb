use log::{ParseLevelError, SetLoggerError};
use lsb_core::{error::StegError, hash::ParseError};
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum LsbError {
    Steg(StegError),
    ParseLevel(ParseLevelError),
    ParseHash(ParseError),
    SetLogger(SetLoggerError),
}

impl std::fmt::Display for LsbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LsbError::Steg(err) => write!(f, "{}", err),
            LsbError::ParseLevel(err) => write!(f, "ParseLevelError: {}", err),
            LsbError::SetLogger(err) => write!(f, "SetLoggerError: {}", err),
            LsbError::ParseHash(err) => write!(f, "ParseHashError: {}", err),
        }
    }
}

impl std::error::Error for LsbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LsbError::Steg(err) => Some(err),
            LsbError::ParseLevel(err) => Some(err),
            LsbError::SetLogger(err) => Some(err),
            LsbError::ParseHash(err) => Some(err),
        }
    }
}

impl From<ParseLevelError> for LsbError {
    fn from(err: ParseLevelError) -> Self {
        LsbError::ParseLevel(err)
    }
}

impl From<SetLoggerError> for LsbError {
    fn from(err: SetLoggerError) -> Self {
        LsbError::SetLogger(err)
    }
}

impl From<StegError> for LsbError {
    fn from(err: StegError) -> Self {
        LsbError::Steg(err)
    }
}

impl From<ParseError> for LsbError {
    fn from(err: ParseError) -> Self {
        LsbError::ParseHash(err)
    }
}

impl From<LsbError> for JsValue {
    fn from(val: LsbError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
