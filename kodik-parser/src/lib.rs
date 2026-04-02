//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

pub mod decoder;
pub mod error;
pub mod parser;
pub mod scraper;
pub mod state;
pub mod util;

pub use error::KodikError;
pub use reqwest::Client;
pub use scraper::{KodikResponse, Link, Links};
pub use state::{KODIK_STATE, KodikState};
