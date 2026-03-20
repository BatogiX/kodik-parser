//! Error types for the Kodik parser library.
use std::string;
use thiserror::Error;

/// Errors from kodik-parser.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum KodikError {
    /// Reqwest HTTP client error.
    #[cfg(feature = "async-impl")]
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    /// Ureq HTTP client error.
    #[cfg(feature = "blocking")]
    #[error("{0}")]
    Ureq(#[from] ureq::Error),

    /// Base64 decoding error.
    #[error("{0}")]
    Decode(#[from] base64::DecodeError),

    /// UTF-8 conversion error.
    #[error("{0}")]
    FromUtf8(#[from] string::FromUtf8Error),

    /// Regex matching error.
    #[error("{0}")]
    Regex(&'static str),

    /// Link cannot be decoded error.
    #[error("link cannot be decoded {0}")]
    LinkCannotBeDecoded(String),
}
