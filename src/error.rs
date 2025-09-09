//! Error types for the Kodik parser library.
use std::{fmt, string};

/// Errors from kodik-parser.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Reqwest HTTP client error.
    #[cfg(feature = "async-impl")]
    Reqwest(reqwest::Error),
    /// Ureq HTTP client error.
    #[cfg(feature = "blocking")]
    Ureq(ureq::Error),
    /// Base64 decoding error.
    Decode(base64::DecodeError),
    /// UTF-8 conversion error.
    FromUtf8(string::FromUtf8Error),
    /// Regex matching error.
    Regex(&'static str),
    /// Link cannot be decoded error.
    LinkCannotBeDecoded(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reqwest(error) => write!(f, "Reqwest: {error}"),
            Self::Ureq(error) => write!(f, "Ureq: {error}"),
            Self::Decode(error) => write!(f, "Decode: {error}"),
            Self::FromUtf8(error) => write!(f, "FromUtf8: {error}"),
            Self::Regex(msg) => write!(f, "Regex: {msg}"),
            Self::LinkCannotBeDecoded(v) => write!(f, "Src: {v} cannot be decoded"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self::Ureq(value)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::Decode(value)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(value: string::FromUtf8Error) -> Self {
        Self::FromUtf8(value)
    }
}
