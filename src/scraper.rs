use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderName, ORIGIN, REFERER, USER_AGENT},
};
use serde::Deserialize;

use crate::{parser::VideoInfo, util};

#[derive(Debug, Deserialize)]
pub struct PlayerResponse {
    pub(crate) advert_script: String,
    pub(crate) domain: String,
    pub(crate) default: u32,
    pub(crate) links: Links,
    pub(crate) ip: String,
}

impl PlayerResponse {
    pub fn advert_script(&self) -> &str {
        &self.advert_script
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn default(&self) -> u32 {
        self.default
    }

    pub fn links(&self) -> &Links {
        &self.links
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }
}

#[derive(Debug, Deserialize)]
pub struct Links {
    #[serde(rename = "360")]
    pub(crate) quality_360: Vec<Link>,
    #[serde(rename = "480")]
    pub(crate) quality_480: Vec<Link>,
    #[serde(rename = "720")]
    pub(crate) quality_720: Vec<Link>,
}

impl Links {
    fn quality_360(&self) -> &[Link] {
        &self.quality_360
    }

    fn quality_480(&self) -> &[Link] {
        &self.quality_480
    }

    fn quality_720(&self) -> &[Link] {
        &self.quality_720
    }
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub(crate) src: String,
    #[serde(rename = "type")]
    pub(crate) mime_type: String,
}

impl Link {
    fn src(&self) -> &str {
        &self.src
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }
}

pub async fn get(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let agent = util::spoof_random_agent();

    let response_text = client.get(url).header(USER_AGENT, agent).send().await?.text().await?;

    Ok(response_text)
}

pub async fn post(client: &Client, url: &str, video_info: VideoInfo<'_>) -> Result<PlayerResponse, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, "https://kodik.info".parse().unwrap());
    headers.insert(
        ACCEPT,
        "application/json, text/javascript, */*; q=0.01".parse().unwrap(),
    );
    headers.insert(REFERER, "https://kodik.info".parse().unwrap());
    headers.insert(USER_AGENT, util::spoof_random_agent().parse().unwrap());
    headers.insert(
        HeaderName::from_static("x-requested-with"),
        "XMLHttpRequest".parse().unwrap(),
    );

    let response_text = client
        .post(url)
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
        let url = "https://kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let response_text = get(&client, url).await.unwrap();
        println!("{response_text:#?}");
    }

    #[tokio::test]
    async fn test_post() {
        let client = Client::new();
        let url = "https://kodik.info/ftor";
        let video_info = VideoInfo::new("seria", "6a2e103e9acf9829c6cba7e69555afb1", "1484069");
        let response_text = post(&client, url, video_info).await.unwrap();
        println!("{response_text:#?}");
    }
}
