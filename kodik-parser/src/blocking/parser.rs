use ureq::Agent;

use crate::{
    blocking::{scraper, util::update_endpoint},
    cache::KODIK_CACHE,
    decoder,
    error::KodikError,
    parser::{VideoInfo, extract_domain},
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
/// let url = "https://kodikplayer.com/some-type/some-id/some-hash/some-quality";
/// let kodik_response = blocking::parse(&agent, url).unwrap();
///
/// let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
/// println!("Link with 720p quality is: {link_720}");
/// # }
/// ```
pub fn parse(agent: &Agent, url: &str) -> Result<KodikResponse, KodikError> {
    let domain = extract_domain(url)?;
    let is_cached = !KODIK_CACHE.endpoint_load().is_empty();

    let video_info_result = VideoInfo::from_url(&url);

    let response_opt = if !is_cached || video_info_result.is_err() {
        Some(scraper::get(agent, url)?)
    } else {
        None
    };

    let video_info = match video_info_result {
        Ok(info) => info,
        Err(_) => VideoInfo::from_response(response_opt.as_ref().unwrap())?,
    };

    if !is_cached {
        log::warn!("Endpoint not found in cache, updating...");
        update_endpoint(agent, domain, response_opt.as_ref().unwrap())?;
    }
    let mut endpoint = KODIK_CACHE.endpoint_load();

    match scraper::post(agent, domain, &endpoint, &video_info) {
        Ok(mut response) => {
            decoder::decode_links(&mut response)?;
            Ok(response)
        }
        Err(err) => {
            if !is_cached {
                return Err(err);
            }

            log::warn!("Endpoint was deprecated in cache, updating...");
            let response = match response_opt {
                Some(ref r) => r,
                None => &scraper::get(agent, url)?,
            };
            update_endpoint(agent, domain, response)?;

            endpoint = KODIK_CACHE.endpoint_load();
            let mut response = scraper::post(agent, domain, &endpoint, &video_info)?;
            decoder::decode_links(&mut response)?;
            Ok(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires network access"]
    fn blocking_parse() {
        let agent = Agent::new_with_defaults();
        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let kodik_response = parse(&agent, url).unwrap();
        println!("{kodik_response:#?}");
    }
}
