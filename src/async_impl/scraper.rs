use reqwest::{
    Client,
    header::{ACCEPT, HeaderName, ORIGIN, REFERER, USER_AGENT},
};

use crate::{parser::VideoInfo, scraper::KodikResponse, util};

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
) -> Result<KodikResponse, reqwest::Error> {
    let kodik_response = client
        .post(format!("https://{domain}{api_endpoint}"))
        .header(ORIGIN, format!("https://{domain}"))
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(REFERER, format!("https://{domain}"))
        .header(USER_AGENT, util::spoof_random_ua())
        .header(HeaderName::from_static("x-requested-with"), "XMLHttpRequest")
        .form(&video_info)
        .send()
        .await?
        .json()
        .await?;

    Ok(kodik_response)
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
        let kodik_response = post(&client, domain, api_endpoint, &video_info).await.unwrap();
        println!("{kodik_response:#?}");
    }
}
