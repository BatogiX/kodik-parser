//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

pub mod cache;
pub mod decoder;
pub mod error;
pub mod parser;
pub mod scraper;
pub mod util;

pub use cache::{KODIK_CACHE, KodikCache};
pub use error::KodikError;
pub use scraper::{KodikResponse, Link, Links};

#[cfg(feature = "async-impl")]
pub mod async_impl;
#[cfg(feature = "async-impl")]
pub use reqwest::Client;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "blocking")]
pub use ureq::Agent;
