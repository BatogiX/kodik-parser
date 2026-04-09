use crate::decoder;
use crate::scraper;
use crate::{KODIK_STATE, Response};
use kodik_utils::KodikError;
use lazy_regex::Lazy;
use regex_lite::Regex;
use reqwest::Client;
use serde::Serialize;

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
    pub(crate) const fn new(r#type: &'a str, hash: &'a str, id: &'a str) -> Self {
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
    pub(crate) fn from_response(html: &'_ str) -> Result<VideoInfo<'_>, KodikError> {
        let from_response_re: &Lazy<Regex> =
            lazy_regex::regex!(r"\.(?P<field>type|hash|id) = '(?P<value>.*?)';");

        log::debug!("Extracting video info from response...");

        let mut r#type = None;
        let mut hash = None;
        let mut id = None;

        for caps in from_response_re.captures_iter(html) {
            match &caps["field"] {
                "type" => {
                    r#type = Some(
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

        let video_info = VideoInfo::new(
            r#type.ok_or(KodikError::Regex("videoInfo.type not found"))?,
            hash.ok_or(KodikError::Regex("videoInfo.hash not found"))?,
            id.ok_or(KodikError::Regex("videoInfo.id not found"))?,
        );
        log::trace!("Extracted video info: {video_info:#?}");

        Ok(video_info)
    }

    /// Extracts video information from URL.
    ///
    /// # Errors
    ///
    /// Returns `KodikError::Regex` if the video information (type, hash, id) is not found in the URL.
    pub(crate) fn from_url(url: &'_ str) -> Result<VideoInfo<'_>, KodikError> {
        let from_url_re: &Lazy<Regex> = lazy_regex::regex!(r"/([^/]+)/(\d+)/([a-z0-9]+)");

        log::debug!("Extracting video info from url...");

        let caps = from_url_re
            .captures(url)
            .ok_or(KodikError::Regex("videoInfo not found"))?;

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
    }
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
pub fn extract_player_url(domain: &str, html: &str) -> Result<String, KodikError> {
    let player_path_re: &Lazy<Regex> = lazy_regex::regex!(
        r#"<script\s*type="text/javascript"\s*src="/(assets/js/app\.player_single[^"]*)""#
    );

    log::debug!("Extracting player url...");
    let player_path = player_path_re
        .captures(html)
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
pub fn extract_endpoint(html: &str) -> Result<String, KodikError> {
    let endpoint_re: &Lazy<Regex> =
        lazy_regex::regex!(r#"\$\.ajax\([^>]+,url:\s*atob\(["\']([\w=]+)["\']\)"#);

    log::debug!("Extracting endpoint...");
    let encoded_endpoint = endpoint_re
        .captures(html)
        .ok_or(KodikError::Regex(
            "there is no api endpoint in player response",
        ))?
        .get(1)
        .unwrap()
        .as_str();

    let endpoint = decoder::decode_base64(encoded_endpoint)?;
    log::trace!("Extracted endpoint: {endpoint}");

    Ok(endpoint)
}

/// Parses a Kodik player page asynchronously and returns structured video stream information.
///
/// This function performs the complete sequence of operations required to
/// fetch, extract, and decode player data from a given Kodik URL:
///
/// 1. **Domain extraction** – Determines the Kodik domain from the provided URL.
/// 2. **HTML retrieval** – Downloads the initial page HTML.
/// 3. **Video info extraction** – Parses the embedded video information payload.
/// 4. **API endpoint resolution** – If not cached, discovers the video info API endpoint.
/// 5. **Player data request** – Sends a POST request to retrieve player data.
/// 6. **Link decoding** – Decrypts and normalizes streaming URLs.
///
/// The function uses a cached `VIDEO_INFO_ENDPOINT` to avoid repeated endpoint lookups.
///
/// # Arguments
/// * `client` – An [`reqwest::Client`] used for making HTTP requests.
/// * `url` – A full Kodik player page URL.
///
/// # Returns
/// A [`KodikResponse`] containing structured player metadata and stream URLs.
///
/// # Errors
/// Returns an error if:
/// - The domain cannot be extracted from the URL.
/// - Network requests fail.
/// - HTML parsing fails due to unexpected format changes.
/// - The API endpoint cannot be found.
/// - Link decoding fails.
///
/// # Example
/// ```no_run
/// use kodik_parser::Client;
///
/// # async fn run() {
/// let client = Client::new();
/// let url = "https://kodikplayer.com/some-type/some-id/some-hash/some-quality";
/// let kodik_response = kodik_parser::parse(&client, url).await.unwrap();
///
/// let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
/// println!("Link with 720p quality is: {link_720}");
/// # }
/// ```
pub async fn parse(client: &Client, url: &str) -> Result<Response, KodikError> {
    let domain = kodik_utils::extract_domain(url)?;
    let mut html = String::new();

    let video_info = if let Ok(video_info) = VideoInfo::from_url(url) {
        video_info
    } else {
        html = scraper::get(client, url).await?;
        VideoInfo::from_response(&html)?
    };

    loop {
        let endpoint = KODIK_STATE.endpoint();

        if !endpoint.is_empty() {
            if let Ok(mut kodik_response) =
                scraper::post(client, domain, &endpoint, &video_info).await
            {
                decoder::decode_links(&mut kodik_response)?;
                return Ok(kodik_response);
            }
            KODIK_STATE.clear_endpoint();
            continue;
        }

        if KODIK_STATE.try_begin_update() {
            log::warn!("Endpoint not found in cache, updating...");
            let fetched;
            let page_html = if html.is_empty() {
                fetched = scraper::get(client, url).await?;
                &fetched
            } else {
                &html
            };
            let player_url = extract_player_url(domain, page_html)?;
            let player_html = scraper::get(client, &player_url).await?;
            let new_endpoint = extract_endpoint(&player_html)?;
            KODIK_STATE.finish_update(new_endpoint);
            continue;
        }

        KODIK_STATE.wait_for_update().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v_info_from_response_test() {
        let expected_video_info =
            VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

        let html = "
  var videoInfo = {};
   vInfo.type = 'video';
   vInfo.hash = '060cab655974d46835b3f4405807acc2';
   vInfo.id = '91873';
</script>";

        let video_info = VideoInfo::from_response(html).unwrap();

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
        let html = r#"
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

        let player_url = extract_player_url(domain, html).unwrap();
        assert_eq!(
            "https://kodikplayer.com/assets/js/app.player_single.0a909e421830a88800354716d562e21654500844d220805110c7cf2092d70b05.js",
            player_url
        );
    }

    #[test]
    fn getting_endpoint() {
        let player_html = r#"==t.secret&&(e.secret=t.secret),userInfo&&"object"===_typeof(userInfo.info)&&(e.info=JSON.stringify(userInfo.info)),void 0!==window.advertTest&&(e.a_test=!0),!0===t.isUpdate&&(e.isUpdate=!0),$.ajax({type:"POST",url:atob("L2Z0b3I="),"#;
        assert_eq!("/ftor", extract_endpoint(player_html).unwrap());
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

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn async_parse() {
        let client = Client::new();
        let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let kodik_response = parse(&client, url).await.unwrap();
        println!("{kodik_response:#?}");
    }
}
