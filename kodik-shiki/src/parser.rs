use kodik_utils::Error;
use lazy_regex::{Regex, regex};

use crate::{TranslationType, scraper::SearchResult};

pub fn extract_id(url: &str) -> Result<&str, Error> {
    let id_re = lazy_regex::regex!(r"/animes?/(?:[a-z])?([0-9]+)(?:-|$|/)");

    id_re
        .captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .ok_or(Error::RegexMatch(format!("id not found in '{url}'")))
}
