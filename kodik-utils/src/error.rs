//! Error types for the Kodik library.
use reqwest::header;
use std::string;
use thiserror::Error as ThisError;

/// Errors from kodik.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Reqwest HTTP client error.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Base64 decoding error.
    #[error(transparent)]
    Decode(#[from] base64::DecodeError),

    /// UTF-8 conversion error.
    #[error(transparent)]
    FromUtf8(#[from] string::FromUtf8Error),

    /// Regex matching error.
    #[error("{0}")]
    RegexMatch(String),

    /// Link cannot be decoded error.
    #[error("link cannot be decoded {0}")]
    LinkCannotBeDecoded(String),

    /// Invaliad header value
    #[error(transparent)]
    InvalidHeaderValue(#[from] header::InvalidHeaderValue),

    /// Not found error.
    #[error("{0}")]
    NotFound(String),

    #[error(transparent)]
    Regex(#[from] lazy_regex::regex::Error),

    #[error(transparent)]
    SerdeYaml(#[from] serde_saphyr::Error),
}
