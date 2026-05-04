mod error;
mod re;
mod scraper;
mod ua;

pub use error::Error;
pub use re::extract_domain;
pub use scraper::{GET, POST};
pub use ua::random_user_agent;
