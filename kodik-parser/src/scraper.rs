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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn link_deserialization() {
        let json = r#"{
            "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
            "type":"application/x-mpegURL"
        }"#;

        let _: Link = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn links_deserialization() {
        let json = r#"{
            "360":[
                {
                    "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                    "type":"application/x-mpegURL"
                }
            ],
            "480":[
                {
                    "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                    "type":"application/x-mpegURL"
                }
            ],
            "720":[
                {
                    "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg83UrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                    "type":"application/x-mpegURL"
                }
            ]
        }"#;

        let _: Links = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn kodik_response_deserialization() {
        let json = r#"{
            "links":{
                "360":[
                    {
                        "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                        "type":"application/x-mpegURL"
                    }
                ],
                "480":[
                    {
                        "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                        "type":"application/x-mpegURL"
                    }
                ],
                "720":[
                    {
                        "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg83UrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                        "type":"application/x-mpegURL"
                    }
                ]
            }
        }"#;

        let _: Response = serde_json::from_str(json).unwrap();
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn get_test() {
        let client = Client::new();
        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        get(&client, url).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn post_test() {
        let client = Client::new();
        let domain = "kodikplayer.com";
        let endpoint = Arc::new("/ftor".to_string());
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");
        post(&client, domain, &endpoint, &video_info).await.unwrap();
    }
}
