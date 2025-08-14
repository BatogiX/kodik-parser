use ureq::Agent;

use crate::{
    blocking::scraper,
    decoder,
    parser::{VIDEO_INFO_ENDPOINT, extract_player_url, extract_video_info, get_api_endpoint, get_domain},
    scraper::PlayerResponse,
};

pub fn parse(agent: &Agent, url: &str) -> Result<PlayerResponse, Box<dyn std::error::Error>> {
    let domain = get_domain(url)?;

    let response_text = scraper::get(agent, url)?;
    let video_info = extract_video_info(&response_text)?;

    if VIDEO_INFO_ENDPOINT.read()?.is_empty() {
        let player_url = extract_player_url(domain, &response_text)?;
        let player_response_text = scraper::get(agent, &player_url)?;
        *VIDEO_INFO_ENDPOINT.write()? = get_api_endpoint(&player_response_text)?;
    }

    let api_endpoint = VIDEO_INFO_ENDPOINT.read()?.clone();
    let mut player_response = scraper::post(agent, domain, &api_endpoint, &video_info)?;

    decoder::decode_links(&mut player_response)?;

    Ok(player_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let agent = Agent::new_with_defaults();
        let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let response = parse(&agent, url).unwrap();
        println!("{response:#?}");
    }
}
