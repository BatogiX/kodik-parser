//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

pub mod decoder;
pub mod error;
pub mod parser;
pub mod scraper;
pub mod state;
pub mod util;

pub use error::KodikError;
pub use scraper::{KodikResponse, Link, Links};
pub use state::{KODIK_STATE, KodikState};

#[cfg(feature = "async-impl")]
pub mod async_impl;
#[cfg(feature = "async-impl")]
pub use reqwest::Client;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "blocking")]
pub use ureq::Agent;
