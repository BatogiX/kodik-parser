use reqwest::Client;

use crate::{
    async_impl::{scraper, util::update_endpoint},
    decoder,
    error::KodikError,
    parser::{VideoInfo, extract_domain},
    scraper::KodikResponse,
    state::KODIK_STATE,
};

/// Parses a Kodik player page asynchronously and returns structured video stream information.
///
/// This function performs the complete sequence of operations required to
/// fetch, extract, and decode player data from a given Kodik URL:
///
/// 1. **Domain extraction** – Determines the Kodik domain from the provided URL.
/// 2. **HTML retrieval** – Downloads the initial page HTML.
/// 3. **Video info extraction** – Parses the embedded video information payload.
/// 4. **API endpoint resolution** – If not cached, discovers the video info API endpoint.
/// 5. **Player data request** – Sends a POST request to retrieve player data.
/// 6. **Link decoding** – Decrypts and normalizes streaming URLs.
///
/// The function uses a cached `VIDEO_INFO_ENDPOINT` to avoid repeated endpoint lookups.
///
/// # Arguments
/// * `client` – An [`reqwest::Client`] used for making HTTP requests.
/// * `url` – A full Kodik player page URL.
///
/// # Returns
/// A [`KodikResponse`] containing structured player metadata and stream URLs.
///
/// # Errors
/// Returns an error if:
/// - The domain cannot be extracted from the URL.
/// - Network requests fail.
/// - HTML parsing fails due to unexpected format changes.
/// - The API endpoint cannot be found.
/// - Link decoding fails.
///
/// # Example
/// ```no_run
/// use reqwest::Client;
/// use kodik_parser::async_impl;
///
/// # async fn run() {
/// let client = Client::new();
/// let url = "https://kodikplayer.com/some-type/some-id/some-hash/some-quality";
/// let kodik_response = async_impl::parse(&client, url).await.unwrap();
///
/// let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
/// println!("Link with 720p quality is: {link_720}");
/// # }
/// ```
pub async fn parse(client: &Client, url: &str) -> Result<KodikResponse, KodikError> {
    let domain = extract_domain(url)?;
    let endpoint = KODIK_STATE.load_endpoint();

    if endpoint.is_empty() {
        log::warn!("Endpoint not found in cache, updating...");
        let html = scraper::get(client, url).await?;
        let video_info = VideoInfo::from_response(&html)?;
        let endpoint = update_endpoint(client, domain, &html).await?;
        let mut kodik_response = scraper::post(client, domain, &endpoint, &video_info).await?;
        decoder::decode_links(&mut kodik_response)?;
        return Ok(kodik_response);
    }

    let video_info = VideoInfo::from_url(url)?;
    if let Ok(mut kodik_response) = scraper::post(client, domain, &endpoint, &video_info).await {
        decoder::decode_links(&mut kodik_response)?;
        Ok(kodik_response)
    } else {
        log::warn!("Endpoint was deprecated in cache, updating...");
        let html = scraper::get(client, url).await?;
        let endpoint = update_endpoint(client, domain, &html).await?;
        let mut kodik_response = scraper::post(client, domain, &endpoint, &video_info).await?;
        decoder::decode_links(&mut kodik_response)?;
        Ok(kodik_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn async_parse() {
        let client = Client::new();
        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let kodik_response = parse(&client, url).await.unwrap();
        println!("{kodik_response:#?}");
    }
}
