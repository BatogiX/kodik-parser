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
