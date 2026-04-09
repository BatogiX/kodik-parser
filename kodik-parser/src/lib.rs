//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

pub(crate) mod decoder;
pub(crate) mod parser;
pub(crate) mod scraper;
pub(crate) mod state;

pub use parser::parse;
pub use scraper::{Link, Links, Response};
pub use state::KODIK_STATE;

pub extern crate reqwest;
