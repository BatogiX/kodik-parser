//! # Kodik Parser library.
//! `kodik-parser` for getting direct links to files from Kodik.

mod decoder;
mod parser;
mod scraper;
mod state;

pub use parser::parse;
pub use scraper::{Link, Links};
pub use state::KODIK_STATE;
