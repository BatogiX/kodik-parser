use crate::parser::VideoInfo;
use kodik_utils::KodikError;
use reqwest::{
    Client,
    header::{ACCEPT, HeaderName, ORIGIN, REFERER, USER_AGENT},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// Response structure for player data containing video links
pub struct Response {
    /// Available video links organized by quality
    pub links: Links,
}

#[derive(Debug, Deserialize)]
/// Container for video links organized by different quality levels
pub struct Links {
    /// Video links for 360p quality
    #[serde(rename = "360")]
    pub quality_360: Vec<Link>,
    /// Video links for 480p quality
    #[serde(rename = "480")]
    pub quality_480: Vec<Link>,
    /// Video links for 720p quality
    #[serde(rename = "720")]
    pub quality_720: Vec<Link>,
}

#[derive(Debug, Deserialize)]
/// Individual video link with source URL and content type
pub struct Link {
    /// Source URL of the video stream
    pub src: String,
    /// MIME type of the video content
    pub r#type: String,
}

pub async fn get(client: &Client, url: &str) -> Result<String, KodikError> {
    let agent = kodik_utils::random_user_agent();

    log::info!("GET to {url}...");

    let html = client
        .get(url)
        .header(USER_AGENT, agent)
        .send()
        .await?
        .text()
        .await?;

    log::trace!("Fetched to {url}, response: {html}");

    Ok(html)
}

pub async fn post(
    client: &Client,
    domain: &str,
    endpoint: &str,
    video_info: &VideoInfo<'_>,
) -> Result<Response, KodikError> {
    let user_agent = kodik_utils::random_user_agent();
    let url = format!("https://{domain}{endpoint}");

    log::info!("POST to {url}...");

    let kodik_response = client
        .post(url)
        .header(ORIGIN, format!("https://{domain}"))
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(REFERER, format!("https://{domain}"))
        .header(USER_AGENT, user_agent)
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
