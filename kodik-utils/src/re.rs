use std::sync::LazyLock;

use regex::Regex;

use crate::KodikError;

/// Extracts the domain from a URL.
///
/// # Errors
///
/// Returns `KodikError::Regex` if no valid domain is found in the URL.
pub fn extract_domain(url: &str) -> Result<&str, KodikError> {
    static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]")
            .expect("valid regex syntax")
    });

    log::debug!("Extracting domain...");
    let domain = DOMAIN_REGEX
        .find(url)
        .ok_or(KodikError::Regex("no valid domain found"))?
        .as_str();
    log::trace!("Extracted domain: {domain}");

    Ok(domain)
}
