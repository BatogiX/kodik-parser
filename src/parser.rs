use std::sync::LazyLock;

use regex::Regex;
use reqwest::Client;
use crate::scraper;

static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]")
        .expect("Pattern is valid")
});

pub async fn parse(client: &Client, url: &str) -> Result<(), regex::Error> {
    let domain = get_domain(url)?;

    // let (data, player_response) = parse_player();

    Ok(())
}

fn get_domain(url: &str) -> Result<&str, regex::Error> {
    if let Some(domain) = DOMAIN_REGEX.find(url) {
        return Ok(domain.as_str());
    }

    Err(regex::Error::Syntax("No valid domain found".to_owned()))
}

async fn parse_player(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let response_text  = scraper::get_with_fake_agent(client, url).await?;

    let data = extract_kodik_data(&response_text);

    Ok(response_text)
}

fn extract_kodik_data(response_text: &str) {
    
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
}
