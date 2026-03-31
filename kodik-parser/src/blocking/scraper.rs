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
    let user_agent = util::random_user_agent();

    log::debug!("GET {url}");

    let html = agent
        .get(url)
        .header(USER_AGENT, user_agent)
        .call()?
        .body_mut()
        .read_to_string()?;

    log::trace!("GET response from {url}: {html:#?}");

    Ok(html)
}

/// Fetches video information from the Kodik API using the provided HTTP agent.
///
/// # Errors
///
/// Returns a `KodikError` if the HTTP request fails or if parsing the response JSON fails.
pub fn post(
    agent: &Agent,
    domain: &str,
    endpoint: &str,
    video_info: &VideoInfo<'_>,
) -> Result<KodikResponse, KodikError> {
    let user_agent = util::random_user_agent();
    let domain = format!("https://{domain}");
    let url = format!("{domain}{endpoint}");
    log::debug!("POST to {url}...");

    let kodik_response = agent
        .post(&url)
        .header(ORIGIN, &domain)
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(REFERER, &domain)
        .header(USER_AGENT, user_agent)
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
        let html = get(&agent, url).unwrap();
        println!("{html:#?}");
    }

    #[test]
    #[ignore = "requires network access"]
    fn post_test() {
        let agent = Agent::new_with_defaults();
        let domain = "kodikplayer.com";
        let endpoint = "/ftor";
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");
        let kodik_response = post(&agent, domain, endpoint, &video_info).unwrap();
        println!("{kodik_response:#?}");
    }
}
