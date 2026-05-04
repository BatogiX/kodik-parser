mod error;
mod re;
mod scraper;
mod ua;

pub use error::Error;
pub use re::extract_domain;
pub use scraper::{build_headers, fetch_as_json, fetch_as_text, post_form_as_json, post_json_as_json};
pub use ua::random_user_agent;
