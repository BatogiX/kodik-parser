use std::{collections::BTreeMap, fmt::Display};

use kodik_utils::Error;
use lazy_regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct KodikApiResponse {
    pub results: Results,
}

impl KodikApiResponse {
    pub fn find_search_result(
        &self,
        translation_title: Option<&str>,
        translation_type: Option<&TranslationType>,
    ) -> Result<&SearchResult, Error> {
        if let Some(translation_title) = translation_title {
            let title_re = Regex::new(&format!(r"(?i).*{translation_title}.*"))?;

            if let Some(result) = self.results.iter().find(|r| title_re.is_match(&r.translation.title)) {
                log::info!("Found translation title '{}'", result.translation.title);
                return Ok(result);
            }

            log::warn!("no video source with title '{translation_title}'");
        } else if let Some(translation_type) = translation_type {
            if let Some(result) = self.results.iter().find(|r| r.translation.r#type == *translation_type) {
                log::info!("Found translation title '{}'", result.translation.title);
                return Ok(result);
            }

            log::warn!("no video source with type '{translation_type}'");
        }

        self.results
            .first()
            .ok_or_else(|| Error::NotFound("no video sources found".to_string()))
    }
}

pub type Results = Vec<SearchResult>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchResult {
    pub link: String,
    pub title: String,
    pub translation: Translation,
    pub seasons: Option<BTreeMap<usize, Season>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Translation {
    pub title: String,
    pub r#type: TranslationType,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationType {
    Voice,
    Subtitles,
}

impl Display for TranslationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Voice => write!(f, "voice"),
            Self::Subtitles => write!(f, "subtitles"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Season {
    pub episodes: BTreeMap<usize, String>,
}
