// #[cfg(test)]
// #[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
// mod tests;

mod anime;
mod parser;
mod scraper;

pub use anime::{Response, UserRate, parse_anime};
pub use scraper::{TranslationType, VideoResult};
