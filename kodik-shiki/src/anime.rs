use kodik_utils::Error;
use reqwest::Client;
use serde::Deserialize;

use crate::{TranslationType, VideoResult};
use crate::{parser, scraper};

/// Retrieves video results for an anime from Kodik.
///
/// # Errors
///
/// Returns `KodikError` if:
/// - The domain cannot be extracted from the URL
/// - The anime ID cannot be extracted from the URL
/// - The Kodik API request fails
/// - No matching video source is found
pub async fn parse_anime(
    client: &Client,
    url: &str,
    cookie: Option<&str>,
    translation_title: Option<&str>,
    translation_type: Option<&TranslationType>,
    episode: Option<usize>,
) -> Result<VideoResult, Error> {
    let domain = kodik_utils::extract_domain(url)?;
    let id = parser::extract_id(url)?;

    let search_response = scraper::get_kodik_videos(client, id).await?;

    let search_result =
        parser::find_search_result(search_response.results, translation_title, translation_type)?;

    if let Some(seasons) = search_result.seasons {
        let last_episode = if let Some(episode) = episode {
            episode
        } else if let Some(cookie) = cookie
            && let Ok(response) = kodik_utils::fetch_as_json::<Response>(
                client,
                &format!("https://{domain}/api/animes/{id}"),
                kodik_utils::build_headers(domain, Some(cookie))?,
            )
            .await
            && let Some(user_rate) = response.user_rate
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

#[derive(Debug, Deserialize)]
pub struct Response {
    user_rate: Option<UserRate>,
}

#[derive(Debug, Deserialize)]
pub struct UserRate {
    episodes: usize,
}
