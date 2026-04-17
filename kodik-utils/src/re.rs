use lazy_regex;

use crate::Error;

/// Extracts the domain from a URL.
///
/// # Errors
///
/// Returns `KodikError::Regex` if no valid domain is found in the URL.
pub fn extract_domain(url: &str) -> Result<&str, Error> {
    let domain_re = lazy_regex::regex!(
        r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]"
    );

    log::debug!("Extracting domain...");

    let domain = domain_re
        .find(url)
        .ok_or(Error::RegexMatch(format!(
            "no valid domain found in '{url}'"
        )))?
        .as_str();

    log::trace!("Extracted domain: {domain}");

    Ok(domain)
}
