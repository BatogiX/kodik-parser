use ureq::{
    Agent,
    http::{
        HeaderName,
        header::{ACCEPT, ORIGIN, REFERER, USER_AGENT},
    },
};

use crate::{error::Error, parser::VideoInfo, scraper::KodikResponse, util};

pub fn get(agent: &Agent, url: &str) -> Result<String, Error> {
    let ua_header = util::spoof_random_ua();

    let response_text = agent
        .get(url)
        .header(USER_AGENT, ua_header)
        .call()?
        .body_mut()
        .read_to_string()?;

    Ok(response_text)
}

pub fn post(
    agent: &Agent,
    domain: &str,
    api_endpoint: &str,
    video_info: &VideoInfo<'_>,
) -> Result<KodikResponse, Error> {
    let kodik_response = agent
        .post(format!("https://{domain}{api_endpoint}"))
        .header(ORIGIN, format!("https://{domain}"))
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(REFERER, format!("https://{domain}"))
        .header(USER_AGENT, util::spoof_random_ua())
        .header(HeaderName::from_static("x-requested-with"), "XMLHttpRequest")
        .send_form(video_info)?
        .body_mut()
        .read_json()?;

    Ok(kodik_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let agent = Agent::new_with_defaults();
        let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let response_text = get(&agent, url).unwrap();
        println!("{response_text:#?}");
    }

    #[test]
    fn test_post() {
        let agent = Agent::new_with_defaults();
        let domain = "kodik.info";
        let api_endpoint = "/ftor";
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");
        let kodik_response = post(&agent, domain, api_endpoint, &video_info).unwrap();
        println!("{kodik_response:#?}");
    }
}
