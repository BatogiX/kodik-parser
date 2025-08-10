use std::{collections::HashMap, error::Error, sync::LazyLock};

use crate::scraper;
use regex::Regex;
use reqwest::Client;

pub async fn parse(client: &Client, url: &str) -> Result<(), regex::Error> {
    let domain = get_domain(url)?;

    // let (data, player_response) = parse_player();

    Ok(())
}

fn get_domain(url: &str) -> Result<&str, regex::Error> {
    static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]").unwrap()
    });

    if let Some(domain) = DOMAIN_REGEX.find(url) {
        return Ok(domain.as_str());
    }

    Err(regex::Error::Syntax("No valid domain found".to_owned()))
}

async fn parse_player(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let response_text = scraper::get_with_fake_agent(client, url).await?;

    let data = extract_video_info(&response_text);

    Ok(response_text)
}

fn extract_video_info(response_text: &str) -> Result<HashMap<&'static str, &str>, regex::Error> {
    static TYPE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"videoInfo\.type = '(.*?)';").unwrap());
    static HASH_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"videoInfo\.hash = '(.*?)';").unwrap());
    static ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"videoInfo\.id = '(.*?)';").unwrap());

    let video_info_type = TYPE_REGEX.captures(response_text);
    let video_info_hash = HASH_REGEX.captures(response_text);
    let video_info_id = ID_REGEX.captures(response_text);

    if video_info_type.is_none() || video_info_hash.is_none() || video_info_id.is_none() {
        return Err(regex::Error::Syntax("videoInfo not found".to_owned()));
    }

    let mut video_info: HashMap<&'static str, &str> = HashMap::with_capacity(6);
    video_info.insert("type", video_info_type.unwrap().get(1).unwrap().as_str());
    video_info.insert("hash", video_info_hash.unwrap().get(1).unwrap().as_str());
    video_info.insert("id", video_info_id.unwrap().get(1).unwrap().as_str());
    video_info.insert("bad_user", "True");
    video_info.insert("info", "{}");
    video_info.insert("cdn_is_working", "True");

    Ok(video_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_with_scheme() {
        let url = "https://kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let result = get_domain(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "kodik.info");
    }

    #[test]
    fn domain_without_scheme() {
        let url = "kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let result = get_domain(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "kodik.info");
    }

    #[test]
    fn kodik_data() {
        let mut expected_video_info = HashMap::with_capacity(6);
        expected_video_info.insert("type", "seria");
        expected_video_info.insert("hash", "6a2e103e9acf9829c6cba7e69555afb1");
        expected_video_info.insert("id", "1484069");
        expected_video_info.insert("bad_user", "True");
        expected_video_info.insert("info", "{}");
        expected_video_info.insert("cdn_is_working", "True");

        let response_text = "
  var videoInfo = {};
   videoInfo.type = 'seria'; 
   videoInfo.hash = '6a2e103e9acf9829c6cba7e69555afb1'; 
   videoInfo.id = '1484069'; 
</script>";

        let video_info = extract_video_info(response_text).unwrap();

        assert_eq!(expected_video_info, video_info);
    }
}
