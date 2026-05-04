use crate::VideoResult;
use crate::scraper::SearchResponse;
use crate::{parser, scraper};
use kodik_utils::Error;
use kodik_utils::GET;
use reqwest::Client;
use serde::Deserialize;

/// Retrieves video results for an anime from Kodik.
///
/// # Errors
///
/// Returns `KodikError` if:
/// - The anime ID cannot be extracted from the URL
/// - The Kodik API request fails
pub async fn resolve_anime(client: &Client, url: &str) -> Result<SearchResponse, Error> {
    let id = parser::extract_id(url)?;
    let search_response: SearchResponse = scraper::get_kodik_videos(client, id).await?;

    Ok(search_response)
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub franchise: Option<String>,
    pub user_rate: Option<UserRate>,
}

#[derive(Debug, Deserialize)]
pub struct UserRate {
    episodes: usize,
}

pub struct VideoMetaData {
    video: VideoResult,
    name: String,
    episodes: Vec<usize>,
}

pub async fn fetch_user_rate(client: &Client, url: &str) -> Result<Option<usize>, Error> {
    let domain = kodik_utils::extract_domain(url)?;
    let id = parser::extract_id(url)?;
    let url = format!("https://{domain}/api/animes/{id}");
    let shiki_api_animes: ShikiApiAnimes = client.fetch_as_json(&url).await?;

    Ok(shiki_api_animes.user_rate.map(|ur| ur.episodes))
}

#[derive(Debug, Deserialize)]
struct ShikiApiAnimes {
    // id: usize,
    // name: String,
    // russian: String,
    // url: String,
    // kind: String,
    // score: String,
    // status: String,
    // episodes: usize,
    // episodes_aired: usize,
    // aired_on: String,
    // released_on: String,
    // rating: String,
    user_rate: Option<UserRate>,
}
