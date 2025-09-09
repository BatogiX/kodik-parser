//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

#![deny(
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unused_imports
)]

mod decoder;
mod parser;
mod scraper;
mod util;

pub use scraper::{KodikResponse, Link, Links};

#[cfg(feature = "async-impl")]
pub mod async_impl;
#[cfg(feature = "blocking")]
pub mod blocking;
pub mod error;
