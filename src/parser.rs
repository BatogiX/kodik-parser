use std::sync::{LazyLock, RwLock};

use crate::decoder;
use regex::Regex;
use serde::Serialize;
pub static VIDEO_INFO_ENDPOINT: RwLock<String> = RwLock::new(String::new());

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct VideoInfo<'a> {
    #[serde(rename = "type")]
    video_type: &'a str,
    hash: &'a str,
    id: &'a str,
    bad_user: &'static str,
    info: &'static str,
    cdn_is_working: &'static str,
}

impl<'a> VideoInfo<'a> {
    pub(crate) const fn new(video_type: &'a str, hash: &'a str, id: &'a str) -> Self {
        Self {
            video_type,
            hash,
            id,
            bad_user: "True",
            info: "{}",
            cdn_is_working: "True",
        }
    }

    fn iter(&'a self) -> std::array::IntoIter<(&'a str, &'a str), 6> {
        [
            ("type", self.video_type),
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
    type IntoIter = std::array::IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub fn get_domain(url: &str) -> Result<&str, Box<dyn std::error::Error>> {
    static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]").unwrap()
    });

    let domain = DOMAIN_REGEX.find(url).ok_or("No valid domain found")?;

    Ok(domain.as_str())
}

pub fn extract_video_info(response_text: &'_ str) -> Result<VideoInfo<'_>, Box<dyn std::error::Error>> {
    static TYPE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"videoInfo\.type = '(.*?)';").unwrap());
    static HASH_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"videoInfo\.hash = '(.*?)';").unwrap());
    static ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"videoInfo\.id = '(.*?)';").unwrap());

    let video_type = TYPE_REGEX
        .captures(response_text)
        .ok_or("videoInfo.type not found")?
        .get(1)
        .unwrap()
        .as_str();

    let hash = HASH_REGEX
        .captures(response_text)
        .ok_or("videoInfo.hash not found")?
        .get(1)
        .unwrap()
        .as_str();

    let id = ID_REGEX
        .captures(response_text)
        .ok_or("videoInfo.id not found")?
        .get(1)
        .unwrap()
        .as_str();

    Ok(VideoInfo::new(video_type, hash, id))
}

pub fn extract_player_url(domain: &str, response_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    static PLAYER_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<script\s*type="text/javascript"\s*src="/(assets/js/app\.player_single[^"]*)""#).unwrap()
    });

    let player_path = PLAYER_PATH_REGEX
        .captures(response_text)
        .ok_or("There is no player path in response text")?
        .get(1)
        .unwrap()
        .as_str();

    Ok(format!("https://{domain}/{player_path}"))
}

pub fn get_api_endpoint(player_response_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    static ENDPOINT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\$\.ajax\([^>]+,url:\s*atob\(["\']([\w=]+)["\']\)"#).unwrap());

    let encoded_api_endpoint = ENDPOINT_REGEX
        .captures(player_response_text)
        .ok_or("There is no api endpoint in player response")?
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
        let url_with_scheme = "https://kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let url_without_scheme = "kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";

        assert_eq!("kodik.info", get_domain(url_with_scheme).unwrap());
        assert_eq!("kodik.info", get_domain(url_without_scheme).unwrap());
    }

    #[test]
    fn test_extract_video_info() {
        let expected_video_info = VideoInfo::new("seria", "6a2e103e9acf9829c6cba7e69555afb1", "1484069");

        let response_text = "
  var videoInfo = {};
   videoInfo.type = 'seria'; 
   videoInfo.hash = '6a2e103e9acf9829c6cba7e69555afb1'; 
   videoInfo.id = '1484069'; 
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
}
