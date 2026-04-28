use crate::decoder;
use crate::{KODIK_STATE, Response};
use kodik_utils::Error;
use reqwest::Client;
use serde::Serialize;

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
/// use kodik_parser::parse;
/// use reqwest::Client;
///
/// # async fn run() {
/// let client = Client::new();
/// let url = "https://kodikplayer.com/some-type/some-id/some-hash/some-quality";
/// let kodik_response = parse(&client, url).await.unwrap();
///
/// let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
/// println!("Link with 720p quality is: {link_720}");
/// # }
/// ```
pub async fn parse(client: &Client, url: &str) -> Result<Response, Error> {
    let domain = kodik_utils::extract_domain(url)?;
    let mut body = String::new();

    let video_info = if let Ok(video_info) = VideoInfo::from_url(url) {
        video_info
    } else {
        log::warn!("video info not found in '{url}', fetching from body...");

        body = kodik_utils::fetch_as_text(client, url, kodik_utils::build_headers(domain)?).await?;

        VideoInfo::from_body(&body)?
    };

    loop {
        let endpoint = KODIK_STATE.endpoint();

        if !endpoint.is_empty() {
            if let Ok(mut kodik_response) = kodik_utils::post_form_as_json::<Response, VideoInfo>(
                client,
                &format!("https://{domain}{endpoint}"),
                kodik_utils::build_headers(domain)?,
                &video_info,
            )
            .await
            {
                kodik_response.decode_links()?;
                return Ok(kodik_response);
            }
            KODIK_STATE.clear_endpoint();
            continue;
        }

        if KODIK_STATE.try_begin_update() {
            log::warn!("Endpoint not found in cache, updating...");
            let fetched;

            let body = if body.is_empty() {
                fetched =
                    kodik_utils::fetch_as_text(client, url, kodik_utils::build_headers(domain)?)
                        .await?;

                &fetched
            } else {
                &body
            };

            let player_body = kodik_utils::fetch_as_text(
                client,
                &extract_player_url(domain, body)?,
                kodik_utils::build_headers(domain)?,
            )
            .await?;

            let new_endpoint = extract_endpoint(&player_body)?;
            KODIK_STATE.finish_update(new_endpoint);
            continue;
        }

        KODIK_STATE.wait_for_update().await;
    }
}

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
    pub(crate) fn from_body(body: &'_ str) -> Result<VideoInfo<'_>, Error> {
        let from_body_re = lazy_regex::regex!(r"\.(?P<field>type|hash|id) = '(?P<value>.*?)';");

        log::debug!("Extracting video info from body...");

        let mut r#type = None;
        let mut hash = None;
        let mut id = None;

        for caps in from_body_re.captures_iter(body) {
            match &caps["field"] {
                "type" => {
                    r#type = Some(
                        caps.name("value")
                            .ok_or(Error::RegexMatch(
                                "videoInfo.type value not found".to_owned(),
                            ))?
                            .as_str(),
                    );
                }
                "hash" => {
                    hash = Some(
                        caps.name("value")
                            .ok_or(Error::RegexMatch(
                                "videoInfo.hash value not found".to_owned(),
                            ))?
                            .as_str(),
                    );
                }
                "id" => {
                    id = Some(
                        caps.name("value")
                            .ok_or(Error::RegexMatch("videoInfo.id value not found".to_owned()))?
                            .as_str(),
                    );
                }
                _ => {}
            }
        }

        let video_info = VideoInfo::new(
            r#type.ok_or(Error::RegexMatch("videoInfo.type not found".to_owned()))?,
            hash.ok_or(Error::RegexMatch("videoInfo.hash not found".to_owned()))?,
            id.ok_or(Error::RegexMatch("videoInfo.id not found".to_owned()))?,
        );

        log::trace!("Extracted video info: {video_info:#?}");
        Ok(video_info)
    }

    /// Extracts video information from URL.
    ///
    /// # Errors
    ///
    /// Returns `KodikError::Regex` if the video information (type, hash, id) is not found in the URL.
    pub(crate) fn from_url(url: &'_ str) -> Result<VideoInfo<'_>, Error> {
        let from_url_re = lazy_regex::regex!(r"/([^/]+)/(\d+)/([a-z0-9]+)");

        log::debug!("Extracting video info from url...");

        let caps = from_url_re
            .captures(url)
            .ok_or(Error::RegexMatch(format!("videoInfo not found in '{url}'")))?;

        let r#type = caps
            .get(1)
            .ok_or(Error::RegexMatch(format!(
                "videoInfo.type not found in '{url}'"
            )))?
            .as_str();
        let id = caps
            .get(2)
            .ok_or(Error::RegexMatch(format!(
                "videoInfo.id not found in '{url}'"
            )))?
            .as_str();
        let hash = caps
            .get(3)
            .ok_or(Error::RegexMatch(format!(
                "videoInfo.hash not found in '{url}'"
            )))?
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
pub fn extract_player_url(domain: &str, body: &str) -> Result<String, Error> {
    let player_path_re = lazy_regex::regex!(
        r#"<script\s*type="text/javascript"\s*src="/(assets/js/app\.player_single[^"]*)""#
    );

    log::debug!("Extracting player url...");

    let player_path = player_path_re
        .captures(body)
        .ok_or(Error::RegexMatch(
            "there is no player path in response".to_owned(),
        ))?
        .get(1)
        .ok_or(Error::RegexMatch(
            "player path capture group not found".to_owned(),
        ))?
        .as_str();

    log::trace!("Extracted player url: {player_path}");
    Ok(format!("https://{domain}/{player_path}"))
}

/// Extracts the API endpoint from player's body.
///
/// # Errors
///
/// Returns `KodikError::Regex` if the API endpoint is not found in the player response text.
///
/// # Panics
///
/// Panics if the regex capture group is not found, which should not happen if the regex is correct.
pub fn extract_endpoint(body: &str) -> Result<String, Error> {
    let endpoint_re = lazy_regex::regex!(r#"\$\.ajax\([^>]+,url:\s*atob\(["\']([\w=]+)["\']\)"#);

    log::debug!("Extracting endpoint...");

    let encoded_endpoint = endpoint_re
        .captures(body)
        .ok_or(Error::RegexMatch(
            "there is no api endpoint in player response".to_owned(),
        ))?
        .get(1)
        .ok_or(Error::RegexMatch(
            "api endpoint capture group not found".to_owned(),
        ))?
        .as_str();

    let endpoint = decoder::decode_base64(encoded_endpoint)?;
    log::trace!("Extracted endpoint: {endpoint}");
    Ok(endpoint)
}
