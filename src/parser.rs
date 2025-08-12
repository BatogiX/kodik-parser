use std::sync::{LazyLock, RwLock};

use crate::{
    decoder,
    scraper::{self, PlayerResponse},
};
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use reqwest::Client;
use serde::Serialize;

static API_ENDPOINT: RwLock<String> = RwLock::new(String::new());

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct VideoInfo<'a> {
    #[serde(rename = "type")]
    video_type: &'a str,
    hash: &'a str,
    id: &'a str,
    bad_user: &'static str,
    info: &'static str,
    cdn_is_working: &'static str,
}

impl<'a> VideoInfo<'a> {
    pub const fn new(video_type: &'a str, hash: &'a str, id: &'a str) -> Self {
        Self {
            video_type,
            hash,
            id,
            bad_user: "True",
            info: "{}",
            cdn_is_working: "True",
        }
    }
}

pub async fn parse(client: &Client, url: &str) -> Result<PlayerResponse, Box<dyn std::error::Error>> {
    let domain = get_domain(url)?;

    let response_text = scraper::get(client, url).await?;
    let video_info = extract_video_info(&response_text)?;

    if API_ENDPOINT.read()?.is_empty() {
        let player_url = extract_player_url(domain, &response_text)?;
        let player_response_text = scraper::get(client, &player_url).await?;
        *API_ENDPOINT.write()? = get_api_endpoint(&player_response_text)?;
    }

    let mut player_response = scraper::post(client, url, video_info).await?;

    decoder::decode_links(&mut player_response)?;

    Ok(player_response)
}

fn get_domain(url: &str) -> Result<&str, Box<dyn std::error::Error>> {
    static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]").unwrap()
    });

    let domain = DOMAIN_REGEX.find(url).ok_or("No valid domain found")?;

    Ok(domain.as_str())
}

fn extract_video_info(response_text: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
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

fn extract_player_url(domain: &str, response_text: &str) -> Result<String, Box<dyn std::error::Error>> {
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

fn get_api_endpoint(player_response_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    static ENDPOINT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\$\.ajax\([^>]+,url:\s*atob\(["\']([\w=]+)["\']\)"#).unwrap());

    let encoded_api_endpoint = ENDPOINT_REGEX
        .captures(player_response_text)
        .ok_or("There is no api endpoint in player response")?
        .get(1)
        .unwrap()
        .as_str();

    Ok(decoder::b64(encoded_api_endpoint)?)
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

    #[tokio::test]
    async fn test_parse() {
        let client = Client::new();
        let url = "https://kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let response = parse(&client, url).await;
        match response {
            Ok(response_ok) => {
                println!("{response_ok:#?}");
            }
            Err(err) => {
                panic!("{err}");
            }
        }
    }
}
