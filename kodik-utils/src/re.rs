use crate::Error;

/// Extracts the domain from a URL.
///
/// # Errors
///
/// Returns `KodikError::Regex` if no valid domain is found in the URL.
pub fn extract_domain(url: &str) -> Result<&str, Error> {
    let domain_re = lazy_regex::regex!(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]");

    log::debug!("Extracting domain...");

    let domain = domain_re
        .find(url)
        .ok_or(Error::RegexMatch(format!("no valid domain found in '{url}'")))?
        .as_str();

    log::trace!("Extracted domain: {domain}");

    Ok(domain)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn getting_domain() {
        let url_with_scheme = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let url_without_scheme = "kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";

        assert_eq!("kodikplayer.com", extract_domain(url_with_scheme).unwrap());
        assert_eq!("kodikplayer.com", extract_domain(url_without_scheme).unwrap());
    }
}
