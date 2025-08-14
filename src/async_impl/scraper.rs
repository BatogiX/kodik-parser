use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderName, ORIGIN, REFERER, USER_AGENT},
};

use crate::{parser::VideoInfo, scraper::PlayerResponse, util};

pub async fn get(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let agent = util::spoof_random_ua();

    let response_text = client.get(url).header(USER_AGENT, agent).send().await?.text().await?;

    Ok(response_text)
}

pub async fn post(
    client: &Client,
    domain: &str,
    api_endpoint: &str,
    video_info: &VideoInfo<'_>,
) -> Result<PlayerResponse, reqwest::Error> {
    let mut headers = HeaderMap::with_capacity(5);
    headers.insert(ORIGIN, format!("https://{domain}").parse().unwrap());
    headers.insert(
        ACCEPT,
        "application/json, text/javascript, */*; q=0.01".parse().unwrap(),
    );
    headers.insert(REFERER, format!("https://{domain}").parse().unwrap());
    headers.insert(USER_AGENT, util::spoof_random_ua().parse().unwrap());
    headers.insert(
        HeaderName::from_static("x-requested-with"),
        "XMLHttpRequest".parse().unwrap(),
    );

    let response_text = client
        .post(format!("https://{domain}{api_endpoint}"))
        .headers(headers)
        .form(&video_info)
        .send()
        .await?
        .json()
        .await?;

    Ok(response_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get() {
        let client = Client::new();
        let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let response_text = get(&client, url).await.unwrap();
        println!("{response_text:#?}");
    }

    #[tokio::test]
    async fn test_post() {
        let client = Client::new();
        let domain = "kodik.info";
        let api_endpoint = "/ftor";
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");
        let response_text = post(&client, domain, api_endpoint, &video_info).await.unwrap();
        println!("{response_text:#?}");
    }
}
