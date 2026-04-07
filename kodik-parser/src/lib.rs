//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

pub(crate) mod decoder;
pub(crate) mod parser;
pub(crate) mod scraper;
pub(crate) mod state;

pub use parser::parse;
pub use reqwest::Client;
pub use scraper::{Link, Links, Response};
pub use state::KODIK_STATE;
