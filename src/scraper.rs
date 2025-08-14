use serde::Deserialize;

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

    pub const fn default(&self) -> u32 {
        self.default
    }

    pub const fn links(&self) -> &Links {
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
