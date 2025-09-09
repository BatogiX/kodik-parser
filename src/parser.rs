use std::{
    array::IntoIter,
    sync::{LazyLock, RwLock},
};

use crate::decoder;
use crate::error::Error;
use regex::Regex;
use serde::Serialize;
pub static VIDEO_INFO_ENDPOINT: RwLock<String> = RwLock::new(String::new());

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct VideoInfo<'a> {
    r#type: &'a str,
    hash: &'a str,
    id: &'a str,
    bad_user: &'static str,
    info: &'static str,
    cdn_is_working: &'static str,
}

impl<'a> VideoInfo<'a> {
    pub const fn new(r#type: &'a str, hash: &'a str, id: &'a str) -> Self {
        Self {
            r#type,
            hash,
            id,
            bad_user: "True",
            info: "{}",
            cdn_is_working: "True",
        }
    }

    fn iter(&'a self) -> IntoIter<(&'a str, &'a str), 6> {
        [
            ("type", self.r#type),
            ("hash", self.hash),
            ("id", self.id),
            ("bad_user", self.bad_user),
            ("info", self.info),
            ("cdn_is_working", self.cdn_is_working),
        ]
        .into_iter()
    }
}

impl<'a> IntoIterator for &'a VideoInfo<'a> {
    type Item = (&'a str, &'a str);
    type IntoIter = IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub fn get_domain(url: &str) -> Result<&str, Error> {
    static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]").unwrap()
    });

    let domain = DOMAIN_REGEX.find(url).ok_or(Error::Regex("No valid domain found"))?;

    Ok(domain.as_str())
}

pub fn extract_video_info(response_text: &'_ str) -> Result<VideoInfo<'_>, Error> {
    static VIDEO_INFO_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"videoInfo\.(?P<field>type|hash|id) = '(?P<value>.*?)';").unwrap());

    let (r#type, hash, id) = {
        let mut video_type = None;
        let mut hash = None;
        let mut id = None;

        for caps in VIDEO_INFO_REGEX.captures_iter(response_text) {
            match &caps["field"] {
                "type" => {
                    video_type = Some(
                        caps.name("value")
                            .ok_or(Error::Regex("videoInfo.type value not found"))?
                            .as_str(),
                    );
                }
                "hash" => {
                    hash = Some(
                        caps.name("value")
                            .ok_or(Error::Regex("videoInfo.hash value not found"))?
                            .as_str(),
                    );
                }
                "id" => {
                    id = Some(
                        caps.name("value")
                            .ok_or(Error::Regex("videoInfo.id value not found"))?
                            .as_str(),
                    );
                }
                _ => {}
            }
        }

        (
            video_type.ok_or(Error::Regex("videoInfo.type not found"))?,
            hash.ok_or(Error::Regex("videoInfo.hash not found"))?,
            id.ok_or(Error::Regex("videoInfo.id not found"))?,
        )
    };

    Ok(VideoInfo::new(r#type, hash, id))
}

pub fn extract_player_url(domain: &str, response_text: &str) -> Result<String, Error> {
    static PLAYER_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<script\s*type="text/javascript"\s*src="/(assets/js/app\.player_single[^"]*)""#).unwrap()
    });

    let player_path = PLAYER_PATH_REGEX
        .captures(response_text)
        .ok_or(Error::Regex("There is no player path in response text"))?
        .get(1)
        .unwrap()
        .as_str();

    Ok(format!("https://{domain}/{player_path}"))
}

pub fn get_api_endpoint(kodik_response_text: &str) -> Result<String, Error> {
    static ENDPOINT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\$\.ajax\([^>]+,url:\s*atob\(["\']([\w=]+)["\']\)"#).unwrap());

    let encoded_api_endpoint = ENDPOINT_REGEX
        .captures(kodik_response_text)
        .ok_or(Error::Regex("There is no api endpoint in player response"))?
        .get(1)
        .unwrap()
        .as_str();

    decoder::b64(encoded_api_endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_domain() {
        let url_with_scheme = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let url_without_scheme = "kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";

        assert_eq!("kodik.info", get_domain(url_with_scheme).unwrap());
        assert_eq!("kodik.info", get_domain(url_without_scheme).unwrap());
    }

    #[test]
    fn test_extract_video_info() {
        let expected_video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

        let response_text = "
  var videoInfo = {};
   videoInfo.type = 'video';
   videoInfo.hash = '060cab655974d46835b3f4405807acc2';
   videoInfo.id = '91873';
</script>";

        let video_info = extract_video_info(response_text).unwrap();

        assert_eq!(expected_video_info, video_info);
    }

    #[test]
    fn test_get_player_url() {
        let domain = "kodik.info";
        let response_text = r#"
  </script>

  <link rel="stylesheet" href="/assets/css/app.player.ffc43caed0b4bc0a9f41f95c06cd8230d49aaf7188dbba5f0770513420541101.css">
  <script type="text/javascript" src="/assets/js/app.player_single.0a909e421830a88800354716d562e21654500844d220805110c7cf2092d70b05.js"></script>
</head>
<body class=" ">
  <div class="main-box">
    <style>
  .resume-button { color: rgba(255, 255, 255, 0.75); }
  .resume-button:hover { background-color: #171717; }
  .resume-button { border-radius: 3px; }
  .active-player .resume-button { border-radius: 3px; }"#;

        let player_url = extract_player_url(domain, response_text).unwrap();
        assert_eq!(
            "https://kodik.info/assets/js/app.player_single.0a909e421830a88800354716d562e21654500844d220805110c7cf2092d70b05.js",
            player_url
        );
    }

    #[test]
    fn test_get_api_endpoint() {
        let player_response_text = r#"==t.secret&&(e.secret=t.secret),userInfo&&"object"===_typeof(userInfo.info)&&(e.info=JSON.stringify(userInfo.info)),void 0!==window.advertTest&&(e.a_test=!0),!0===t.isUpdate&&(e.isUpdate=!0),$.ajax({type:"POST",url:atob("L2Z0b3I="),"#;
        assert_eq!("/ftor", get_api_endpoint(player_response_text).unwrap());
    }

    #[test]
    fn test_video_info_serialize() {
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

        let serialized = serde_json::to_string(&video_info).unwrap();
        assert_eq!(
            r#"{"type":"video","hash":"060cab655974d46835b3f4405807acc2","id":"91873","bad_user":"True","info":"{}","cdn_is_working":"True"}"#,
            serialized
        );
    }
}
