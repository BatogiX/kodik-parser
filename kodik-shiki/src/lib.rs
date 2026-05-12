// #[cfg(test)]
// #[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
// mod tests;

mod anime;
mod error;
mod models;
pub(crate) mod parser;
mod related;
pub(crate) mod scraper;

pub use anime::{fetch_shiki_api_animes, fetch_user_rate};
pub use related::fetch_not_anime_ids;
pub use scraper::{VideoResult, fetch_kodik_videos};

pub use error::Error;
pub(crate) use error::Result;
pub use models::*;
pub use parser::extract_id;
