use ureq::{
    Agent,
    http::{
        HeaderName,
        header::{ACCEPT, ORIGIN, REFERER, USER_AGENT},
    },
};

use crate::{error::KodikError, parser::VideoInfo, scraper::KodikResponse, util};

/// Fetches the content of a URL using the provided HTTP agent.
///
/// # Errors
///
/// Returns a `KodikError` if the HTTP request fails or if reading the response body fails.
pub fn get(agent: &Agent, url: &str) -> Result<String, KodikError> {
    let ua_header = util::spoof_random_ua();

    log::debug!("GET {url}");

    let response_text = agent
        .get(url)
        .header(USER_AGENT, ua_header)
        .call()?
        .body_mut()
        .read_to_string()?;

    log::trace!("GET response from {url}: {response_text:#?}");

    Ok(response_text)
}

/// Fetches video information from the Kodik API using the provided HTTP agent.
///
/// # Errors
///
/// Returns a `KodikError` if the HTTP request fails or if parsing the response JSON fails.
pub fn post(
    agent: &Agent,
    domain: &str,
    api_endpoint: &str,
    video_info: &VideoInfo<'_>,
) -> Result<KodikResponse, KodikError> {
    let ua_header = util::spoof_random_ua();
    let url = format!("https://{domain}{api_endpoint}");
    log::debug!("POST to {url}...");

    let kodik_response = agent
        .post(format!("https://{domain}{api_endpoint}"))
        .header(ORIGIN, format!("https://{domain}"))
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(REFERER, format!("https://{domain}"))
        .header(USER_AGENT, ua_header)
        .header(
            HeaderName::from_static("x-requested-with"),
            "XMLHttpRequest",
        )
        .send_form(video_info)?
        .body_mut()
        .read_json()?;

    log::trace!("POST response from {url}:\n{kodik_response:#?}");

    Ok(kodik_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires network access"]
    fn get_test() {
        let agent = Agent::new_with_defaults();
        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let response_text = get(&agent, url).unwrap();
        println!("{response_text:#?}");
    }

    #[test]
    #[ignore = "requires network access"]
    fn post_test() {
        let agent = Agent::new_with_defaults();
        let domain = "kodikplayer.com";
        let api_endpoint = "/ftor";
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");
        let kodik_response = post(&agent, domain, api_endpoint, &video_info).unwrap();
        println!("{kodik_response:#?}");
    }
}
