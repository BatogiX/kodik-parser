use crate::{KodikApiResponse, ShikiApiAnimes, VideoResult, parser, scraper};
use kodik_utils::{Client, Error, GET};

/// Retrieves video results for an anime from Kodik.
///
/// # Errors
///
/// Returns `KodikError` if:
/// - The anime ID cannot be extracted from the URL
/// - The Kodik API request fails
pub async fn resolve_anime(client: &Client, url: &str) -> Result<KodikApiResponse, Error> {
    let id = parser::extract_id(url)?;
    let search_response: KodikApiResponse = scraper::get_kodik_videos(client, id).await?;

    Ok(search_response)
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

pub async fn fetch_shiki_api_animes(client: &Client, url: &str) -> Result<ShikiApiAnimes, Error> {
    let url = url.replace("animes", "api/animes");
    let shiki_api_animes = client.fetch_as_json(&url).await?;

    Ok(shiki_api_animes)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_shiki_api_animes_test() {
        let client = Client::new();
        let url = "https://shikimori.net/animes/33";

        dbg!(fetch_shiki_api_animes(&client, url).await.unwrap());
    }
}
