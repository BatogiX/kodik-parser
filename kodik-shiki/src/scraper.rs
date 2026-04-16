use std::{collections::BTreeMap, fmt::Debug};

use kodik_utils::{KodikError, extract_domain};
use reqwest::{
    Client,
    header::{ACCEPT, COOKIE, HOST, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Deserialize, de::DeserializeOwned};

use crate::parser::extract_id;

pub async fn get_json<T: DeserializeOwned + Debug>(
    client: &Client,
    url: &str,
    headers: HeaderMap,
) -> Result<T, KodikError> {
    let agent = kodik_utils::random_user_agent();

    log::debug!("GET to {url}...");

    let resp = client
        .get(url)
        .header(USER_AGENT, agent)
        .headers(headers)
        .send()
        .await?
        .json()
        .await?;

    log::trace!("Fetched to {url}, response: {resp:#?}");

    Ok(resp)
}

fn build_headers(host: &str, with_cookie: Option<&str>) -> Result<HeaderMap, KodikError> {
    let mut headers = HeaderMap::with_capacity(if with_cookie.is_some() { 3 } else { 2 });

    headers.insert(HOST, HeaderValue::from_str(host)?);
    headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);

    if let Some(cookie) = with_cookie {
        let mut cookie_header = HeaderValue::from_str(cookie)?;
        cookie_header.set_sensitive(true);
        headers.insert(COOKIE, cookie_header);
    }

    Ok(headers)
}

#[derive(Debug, Deserialize)]
pub struct Response {
    user_rate: Option<UserRate>,
}

#[derive(Debug, Deserialize)]
pub struct UserRate {
    episodes: usize,
}

pub async fn get_user_rate(
    client: &Client,
    domain: &str,
    id: &str,
    cookie: &str,
) -> Result<Option<UserRate>, KodikError> {
    let url = format!("https://{domain}/api/animes/{id}");
    let headers = build_headers(domain, Some(cookie))?;

    let resp: Response = get_json(client, &url, headers).await?;
    Ok(resp.user_rate)
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchResult {
    pub link: String,
    pub translation: Translation,
    pub seasons: Option<BTreeMap<usize, Season>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Translation {
    pub title: String,
    pub r#type: TranslationType,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TranslationType {
    Voice,
    Subtitles,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Season {
    episodes: BTreeMap<usize, String>,
}

#[derive(Debug)]
pub enum VideoResult {
    Episodes(Vec<String>),
    Film(String),
}

pub async fn get_kodik_videos(client: &Client, id: &str) -> Result<SearchResponse, KodikError> {
    let token = env!("KODIK_TOKEN");
    let url = format!(
        "https://kodik-api.com/search?token={token}&shikimori_id={id}&with_seasons=true&with_episodes=true"
    );

    let headers = build_headers("kodik-api.com", None)?;
    get_json(client, &url, headers).await
}

pub fn find_search_result(
    results: Vec<SearchResult>,
    translation_title: Option<&str>,
    translation_type: Option<&TranslationType>,
) -> Result<SearchResult, KodikError> {
    if let Some(title) = translation_title {
        results
            .into_iter()
            .find(|r| r.translation.title == title)
            .ok_or(KodikError::NotFound(format!(
                "no video source with title '{title}'"
            )))
    } else if let Some(r#type) = translation_type {
        results
            .into_iter()
            .find(|r| r.translation.r#type == *r#type)
            .ok_or(KodikError::NotFound(
                "no video source with matching type".to_string(),
            ))
    } else {
        results
            .into_iter()
            .next()
            .ok_or(KodikError::NotFound("no video sources found".to_string()))
    }
}

/// Retrieves video results for an anime from Kodik.
///
/// # Errors
///
/// Returns `KodikError` if:
/// - The domain cannot be extracted from the URL
/// - The anime ID cannot be extracted from the URL
/// - The Kodik API request fails
/// - No matching video source is found
pub async fn run(
    client: &Client,
    url: &str,
    cookie: Option<&str>,
    translation_title: Option<&str>,
    translation_type: Option<&TranslationType>,
    episode: Option<usize>,
) -> Result<VideoResult, KodikError> {
    let domain = extract_domain(url)?;
    let id = extract_id(url)?;

    let search_response = get_kodik_videos(client, id).await?;

    let search_result =
        find_search_result(search_response.results, translation_title, translation_type)?;

    if let Some(seasons) = search_result.seasons {
        let last_episode = if let Some(episode) = episode {
            episode
        } else if let Some(cookie) = cookie
            && let Ok(Some(user_rate)) = get_user_rate(client, domain, id, cookie).await
        {
            user_rate.episodes
        } else {
            0
        };

        let (_, season) = seasons
            .into_iter()
            .next()
            .ok_or(KodikError::NotFound("no season found".to_string()))?;

        let episodes = season
            .episodes
            .into_iter()
            .skip(last_episode)
            .map(|(_, ep)| ep)
            .collect();

        Ok(VideoResult::Episodes(episodes))
    } else {
        Ok(VideoResult::Film(search_result.link))
    }
}
