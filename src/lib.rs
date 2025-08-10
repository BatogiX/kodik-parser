use regex::Regex;

pub fn parse(url: &str) -> Result<(), regex::Error> {
    let domain = get_domain(url)?;

    // let (data, player_response) = parse_player();

    Ok(())
}

fn get_domain(url: &str) -> Result<&str, regex::Error> {
    let domain_regex: Regex =
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]")
            .expect("Pattern is valid");

    if let Some(domain) = domain_regex.find(url) {
        return Ok(domain.as_str());
    }

    Err(regex::Error::Syntax("No valid domain found".to_owned()))
}

fn parse_player() {

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
