use std::sync::Arc;

use reqwest::{
    Client,
    header::{ACCEPT, HeaderName, ORIGIN, REFERER, USER_AGENT},
};

use crate::{error::KodikError, parser::VideoInfo, scraper::KodikResponse, util};

pub async fn get(client: &Client, url: &str) -> Result<String, KodikError> {
    let agent = util::spoof_random_ua();

    log::debug!("Fetching response...");

    let response_text = client
        .get(url)
        .header(USER_AGENT, agent)
        .send()
        .await?
        .text()
        .await?;

    log::trace!("Fetched to {url}, response: {response_text}");

    Ok(response_text)
}

pub async fn post(
    client: &Client,
    domain: &str,
    endpoint: &Arc<String>,
    video_info: &VideoInfo<'_>,
) -> Result<KodikResponse, KodikError> {
    let ua_header = util::spoof_random_ua();

    log::debug!("Posting to endpoint...");

    let kodik_response = client
        .post(format!("https://{domain}{endpoint}"))
        .header(ORIGIN, format!("https://{domain}"))
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(REFERER, format!("https://{domain}"))
        .header(USER_AGENT, ua_header)
        .header(
            HeaderName::from_static("x-requested-with"),
            "XMLHttpRequest",
        )
        .form(&video_info)
        .send()
        .await?
        .json()
        .await?;

    log::trace!("POST Response: {kodik_response:#?}");

    Ok(kodik_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_test() {
        let client = Client::new();
        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        get(&client, url).await.unwrap();
    }

    #[tokio::test]
    async fn post_test() {
        let client = Client::new();
        let domain = "kodikplayer.com";
        let endpoint = Arc::new("/ftor".to_string());
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");
        post(&client, domain, &endpoint, &video_info).await.unwrap();
    }
}
