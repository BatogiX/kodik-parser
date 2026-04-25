#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

mod anime;
mod parser;
mod scraper;

pub use anime::*;
pub use parser::*;
pub use scraper::*;
