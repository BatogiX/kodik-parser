use std::{array::IntoIter, sync::LazyLock};

use crate::decoder;
use crate::error::KodikError;
use arc_swap::ArcSwap;
use regex::Regex;
use serde::Serialize;

pub static VIDEO_INFO_ENDPOINT: LazyLock<ArcSwap<String>> =
    LazyLock::new(|| ArcSwap::from_pointee(String::new()));

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
    #[must_use]
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

    /// Extracts video information from response text.
    ///
    /// # Errors
    ///
    /// Returns `KodikError::Regex` if any of the required video fields (type, hash, id) are not found in the response text.
    pub fn from_response(response_text: &'_ str) -> Result<VideoInfo<'_>, KodikError> {
        static FROM_RESPONSE_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\.(?P<field>type|hash|id) = '(?P<value>.*?)';")
                .expect("valid regex syntax")
        });

        log::debug!("Extracting video info from response...");
        let (r#type, hash, id) = {
            let mut video_type = None;
            let mut hash = None;
            let mut id = None;

            for caps in FROM_RESPONSE_RE.captures_iter(response_text) {
                match &caps["field"] {
                    "type" => {
                        video_type = Some(
                            caps.name("value")
                                .ok_or(KodikError::Regex("videoInfo.type value not found"))?
                                .as_str(),
                        );
                    }
                    "hash" => {
                        hash = Some(
                            caps.name("value")
                                .ok_or(KodikError::Regex("videoInfo.hash value not found"))?
                                .as_str(),
                        );
                    }
                    "id" => {
                        id = Some(
                            caps.name("value")
                                .ok_or(KodikError::Regex("videoInfo.id value not found"))?
                                .as_str(),
                        );
                    }
                    _ => {}
                }
            }

            (
                video_type.ok_or(KodikError::Regex("videoInfo.type not found"))?,
                hash.ok_or(KodikError::Regex("videoInfo.hash not found"))?,
                id.ok_or(KodikError::Regex("videoInfo.id not found"))?,
            )
        };

        let video_info = VideoInfo::new(r#type, hash, id);
        log::trace!("Extracted video info: {video_info:#?}");

        Ok(video_info)
    }

    /// Extracts video information from URL.
    ///
    /// # Errors
    ///
    /// Returns `KodikError::Regex` if the video information (type, hash, id) is not found in the URL.
    pub fn from_url(url: &'_ str) -> Result<VideoInfo<'_>, KodikError> {
        static FROM_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"/([^/]+)/(\d+)/([a-z0-9]+)").expect("valid regex syntax")
        });

        log::debug!("Extracting video info from url...");
        if let Some(caps) = FROM_URL_RE.captures(url) {
            let r#type = caps
                .get(1)
                .ok_or(KodikError::Regex("videoInfo.type not found"))?
                .as_str();
            let id = caps
                .get(2)
                .ok_or(KodikError::Regex("videoInfo.id not found"))?
                .as_str();
            let hash = caps
                .get(3)
                .ok_or(KodikError::Regex("videoInfo.hash not found"))?
                .as_str();

            Ok(VideoInfo::new(r#type, hash, id))
        } else {
            Err(KodikError::Regex("videoInfo not found"))
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

/// Extracts the domain from a URL.
///
/// # Errors
///
/// Returns `KodikError::Regex` if no valid domain is found in the URL.
pub fn extract_domain(url: &str) -> Result<&str, KodikError> {
    static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]")
            .expect("valid regex syntax")
    });

    log::debug!("Extracting domain...");
    let domain = DOMAIN_REGEX
        .find(url)
        .ok_or(KodikError::Regex("no valid domain found"))?
        .as_str();
    log::trace!("Extracted domain: {domain}");

    Ok(domain)
}

/// Extracts the player URL from response text.
///
/// # Errors
///
/// Returns `KodikError::Regex` if the player path is not found in the response text.
///
/// # Panics
///
/// Panics if the regex capture group is not found, which should not happen if the regex is correct.
pub fn extract_player_url(domain: &str, response_text: &str) -> Result<String, KodikError> {
    static PLAYER_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"<script\s*type="text/javascript"\s*src="/(assets/js/app\.player_single[^"]*)""#,
        )
        .expect("valid regex syntax")
    });

    log::debug!("Extracting player url...");
    let player_path = PLAYER_PATH_REGEX
        .captures(response_text)
        .ok_or(KodikError::Regex(
            "there is no player path in response text",
        ))?
        .get(1)
        .unwrap()
        .as_str();
    log::trace!("Extracted player url: {player_path}");

    Ok(format!("https://{domain}/{player_path}"))
}

/// Extracts the API endpoint from player response text.
///
/// # Errors
///
/// Returns `KodikError::Regex` if the API endpoint is not found in the player response text.
///
/// # Panics
///
/// Panics if the regex capture group is not found, which should not happen if the regex is correct.
pub fn extract_api_endpoint(response_text: &str) -> Result<String, KodikError> {
    static ENDPOINT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"\$\.ajax\([^>]+,url:\s*atob\(["\']([\w=]+)["\']\)"#)
            .expect("valid regex syntax")
    });

    log::debug!("Extracting endpoint...");
    let encoded_api_endpoint = ENDPOINT_REGEX
        .captures(response_text)
        .ok_or(KodikError::Regex(
            "there is no api endpoint in player response",
        ))?
        .get(1)
        .unwrap()
        .as_str();

    let endpoint = decoder::b64(encoded_api_endpoint)?;
    log::trace!("Extracted endpoint: {endpoint}");

    Ok(endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getting_domain() {
        let url_with_scheme =
            "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let url_without_scheme =
            "kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";

        assert_eq!("kodikplayer.com", extract_domain(url_with_scheme).unwrap());
        assert_eq!(
            "kodikplayer.com",
            extract_domain(url_without_scheme).unwrap()
        );
    }

    #[test]
    fn v_info_from_response_test() {
        let expected_video_info =
            VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

        let response_text = "
  var videoInfo = {};
   vInfo.type = 'video';
   vInfo.hash = '060cab655974d46835b3f4405807acc2';
   vInfo.id = '91873';
</script>";

        let video_info = VideoInfo::from_response(response_text).unwrap();

        assert_eq!(expected_video_info, video_info);
    }

    #[test]
    fn v_info_from_url_test() {
        let expected_video_info =
            VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2";
        let video_info = VideoInfo::from_url(url).unwrap();

        assert_eq!(expected_video_info, video_info);
    }

    #[test]
    fn getting_player_url() {
        let domain = "kodikplayer.com";
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
            "https://kodikplayer.com/assets/js/app.player_single.0a909e421830a88800354716d562e21654500844d220805110c7cf2092d70b05.js",
            player_url
        );
    }

    #[test]
    fn getting_api_endpoint() {
        let player_response_text = r#"==t.secret&&(e.secret=t.secret),userInfo&&"object"===_typeof(userInfo.info)&&(e.info=JSON.stringify(userInfo.info)),void 0!==window.advertTest&&(e.a_test=!0),!0===t.isUpdate&&(e.isUpdate=!0),$.ajax({type:"POST",url:atob("L2Z0b3I="),"#;
        assert_eq!("/ftor", extract_api_endpoint(player_response_text).unwrap());
    }

    #[test]
    fn video_info_serializing() {
        let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

        let serialized = serde_json::to_string(&video_info).unwrap();
        assert_eq!(
            r#"{"type":"video","hash":"060cab655974d46835b3f4405807acc2","id":"91873","bad_user":"True","info":"{}","cdn_is_working":"True"}"#,
            serialized
        );
    }
}
