use lazy_regex;

use crate::KodikError;

/// Extracts the domain from a URL.
///
/// # Errors
///
/// Returns `KodikError::Regex` if no valid domain is found in the URL.
pub fn extract_domain(url: &str) -> Result<&str, KodikError> {
    let domain_re = lazy_regex::regex!(
        r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]"
    );

    log::debug!("Extracting domain...");

    let domain = domain_re
        .find(url)
        .ok_or(KodikError::Regex("no valid domain found"))?
        .as_str();

    log::trace!("Extracted domain: {domain}");

    Ok(domain)
}
