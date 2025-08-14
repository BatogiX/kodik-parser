use reqwest::Client;

use crate::{
    async_impl::scraper,
    decoder,
    parser::{VIDEO_INFO_ENDPOINT, extract_player_url, extract_video_info, get_api_endpoint, get_domain},
    scraper::PlayerResponse,
};

pub async fn parse(client: &Client, url: &str) -> Result<PlayerResponse, Box<dyn std::error::Error>> {
    let domain = get_domain(url)?;

    let response_text = scraper::get(client, url).await?;
    let video_info = extract_video_info(&response_text)?;

    if VIDEO_INFO_ENDPOINT.read()?.is_empty() {
        let player_url = extract_player_url(domain, &response_text)?;
        let player_response_text = scraper::get(client, &player_url).await?;
        *VIDEO_INFO_ENDPOINT.write()? = get_api_endpoint(&player_response_text)?;
    }

    let api_endpoint = VIDEO_INFO_ENDPOINT.read()?.clone();
    let mut player_response = scraper::post(client, domain, &api_endpoint, &video_info).await?;

    decoder::decode_links(&mut player_response)?;

    Ok(player_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse() {
        let client = Client::new();
        let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let response = parse(&client, url).await.unwrap();
        println!("{response:#?}");
    }
}
