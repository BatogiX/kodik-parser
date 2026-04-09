#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

pub mod error;
pub mod re;
pub mod ua;

pub use error::KodikError;
pub use re::extract_domain;
pub use ua::random_user_agent;
