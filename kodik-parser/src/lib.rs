//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

mod decoder;
mod error;
mod parser;
mod scraper;
mod util;

pub use decoder::SHIFT;
pub use error::KodikError;
pub use parser::VIDEO_INFO_ENDPOINT;
pub use scraper::{KodikResponse, Link, Links};

#[cfg(feature = "async-impl")]
pub mod async_impl;

#[cfg(feature = "blocking")]
pub mod blocking;
