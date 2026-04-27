// #[cfg(test)]
// #[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
// mod tests;

mod anime;
pub(crate) mod parser;
mod related;
pub(crate) mod scraper;

pub use anime::{Response, UserRate, resolve_anime};
pub use scraper::{TranslationType, VideoResult};
