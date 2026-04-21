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

pub fn find_search_result(
    mut results: Vec<SearchResult>,
    translation_title: Option<&str>,
    translation_type: Option<&TranslationType>,
) -> Result<SearchResult, Error> {
    if let Some(title) = translation_title {
        let title_re = Regex::new(&format!(r"(?i).*{}.*", regex::escape(title)))?;

        let found_idx = results
            .iter()
            .position(|r| title_re.is_match(&r.translation.title));

        match found_idx {
            Some(idx) => {
                let result = results.remove(idx);
                log::info!("Found translation title '{}'", result.translation.title);
                Ok(result)
            }
            None => translation_type.map_or_else(
                || {
                    Err(Error::NotFound(format!(
                        "no video source with title '{title}'"
                    )))
                },
                |r#type| {
                    results
                        .into_iter()
                        .find(|r| r.translation.r#type == *r#type)
                        .ok_or(Error::NotFound(
                            "no video source with matching type".to_string(),
                        ))
                },
            ),
        }
    } else if let Some(search_type) = translation_type {
        results
            .into_iter()
            .find(|r| r.translation.r#type == *search_type)
            .ok_or(Error::NotFound(
                "no video source with matching type".to_string(),
            ))
    } else {
        results
            .into_iter()
            .next()
            .ok_or(Error::NotFound("no video sources found".to_string()))
    }
}
