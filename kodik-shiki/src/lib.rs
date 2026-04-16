#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

mod parser;
mod scraper;
pub use scraper::run;
pub use scraper::{TranslationType, VideoResult};
