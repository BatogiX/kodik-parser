//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

pub(crate) mod decoder;
pub(crate) mod error;
pub(crate) mod parser;
pub(crate) mod scraper;
pub(crate) mod state;
pub(crate) mod util;

pub use error::KodikError;
pub use parser::parse;
pub use reqwest::Client;
pub use scraper::{KodikResponse, Link, Links};
pub use state::KODIK_STATE;
