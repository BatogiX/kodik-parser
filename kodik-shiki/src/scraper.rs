use std::{collections::BTreeMap, fmt::Debug};

use kodik_utils::Error;
use lazy_regex::{Regex, regex};
use reqwest::{
    Client,
    header::{ACCEPT, COOKIE, HOST, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Deserialize, de::DeserializeOwned};

use crate::parser::extract_id;

#[derive(Debug, Deserialize)]
pub struct Response {
    user_rate: Option<UserRate>,
}

#[derive(Debug, Deserialize)]
pub struct UserRate {
    episodes: usize,
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

pub async fn get_user_rate(
    client: &Client,
    domain: &str,
    id: &str,
    cookie: &str,
) -> Result<Option<UserRate>, Error> {
    let url = format!("https://{domain}/api/animes/{id}");
    let headers = build_headers(domain, Some(cookie))?;

    let resp: Response = get_json(client, &url, headers).await?;
    Ok(resp.user_rate)
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

pub async fn get_json<T: DeserializeOwned + Debug>(
    client: &Client,
    url: &str,
    headers: HeaderMap,
) -> Result<T, Error> {
    let agent = kodik_utils::random_user_agent();

    log::info!("GET to {url}...");

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

fn build_headers(host: &str, with_cookie: Option<&str>) -> Result<HeaderMap, Error> {
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

pub async fn get_kodik_videos(client: &Client, id: &str) -> Result<SearchResponse, Error> {
    let token = env!("KODIK_TOKEN");
    let url = format!(
        "https://kodik-api.com/search?token={token}&shikimori_id={id}&with_seasons=true&with_episodes=true"
    );

    let headers = build_headers("kodik-api.com", None)?;
    get_json(client, &url, headers).await
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
) -> Result<VideoResult, Error> {
    let domain = kodik_utils::extract_domain(url)?;
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
            .next_back()
            .ok_or(Error::NotFound("no season found".to_string()))?;

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
