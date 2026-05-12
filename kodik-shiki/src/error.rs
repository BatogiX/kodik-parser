//! Error types for the Kodik-Shiki library.
use std::num::ParseIntError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors from shiki.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    KodikUtils(#[from] kodik_utils::Error),

    #[error("invalid anime id `{value}`")]
    InvalidAnimeId {
        value: String,
        #[source]
        source: ParseIntError,
    },
}
