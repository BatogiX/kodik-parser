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
    #[must_use]
    pub fn advert_script(&self) -> &str {
        &self.advert_script
    }

    #[must_use]
    pub fn domain(&self) -> &str {
        &self.domain
    }

    #[must_use]
    pub const fn default(&self) -> u32 {
        self.default
    }

    #[must_use]
    pub const fn links(&self) -> &Links {
        &self.links
    }

    #[must_use]
    pub fn ip(&self) -> &str {
        &self.ip
    }
}

#[allow(clippy::struct_field_names)]
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
    #[must_use]
    pub fn quality_360(&self) -> &[Link] {
        &self.quality_360
    }

    #[must_use]
    pub fn quality_480(&self) -> &[Link] {
        &self.quality_480
    }

    #[must_use]
    pub fn quality_720(&self) -> &[Link] {
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
    #[must_use]
    pub fn src(&self) -> &str {
        &self.src
    }

    #[must_use]
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }
}
