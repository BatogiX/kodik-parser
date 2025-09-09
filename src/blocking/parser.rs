use std::sync;

use ureq::Agent;

use crate::{
    blocking::scraper,
    decoder,
    error::Error,
    parser::{VIDEO_INFO_ENDPOINT, extract_player_url, extract_video_info, get_api_endpoint, get_domain},
    scraper::KodikResponse,
};

/// Parses a Kodik player page synchronously and returns structured video stream information.
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
/// * `agent` – An [`ureq::Agent`] used for making HTTP requests.
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
/// use ureq::Agent;
/// use kodik_parser::blocking;
///
/// # fn run() {
/// let agent = Agent::new_with_defaults();
/// let url = "https://kodik.info/some-type/some-id/some-hash/some-quality";
/// let kodik_response = blocking::parse(&agent, url).unwrap();
///
/// let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
/// println!("Link with 720p quality is: {}", link_720);
/// # }
/// ```
pub fn parse(agent: &Agent, url: &str) -> Result<KodikResponse, Error> {
    let domain = get_domain(url)?;
    let response_text = scraper::get(agent, url)?;
    let video_info = extract_video_info(&response_text)?;

    let api_endpoint = {
        if VIDEO_INFO_ENDPOINT
            .read()
            .unwrap_or_else(sync::PoisonError::into_inner)
            .is_empty()
        {
            let player_url = extract_player_url(domain, &response_text)?;
            let player_response_text = scraper::get(agent, &player_url)?;
            *VIDEO_INFO_ENDPOINT
                .write()
                .unwrap_or_else(sync::PoisonError::into_inner) = get_api_endpoint(&player_response_text)?;
        }

        VIDEO_INFO_ENDPOINT
            .read()
            .unwrap_or_else(sync::PoisonError::into_inner)
            .clone()
    };

    let mut kodik_response = scraper::post(agent, domain, &api_endpoint, &video_info)?;
    decoder::decode_links(&mut kodik_response)?;

    Ok(kodik_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let agent = Agent::new_with_defaults();
        let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let kodik_response = parse(&agent, url).unwrap();
        println!("{kodik_response:#?}");
    }
}
