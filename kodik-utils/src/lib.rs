pub mod error;
pub use error::KodikError;
use regex::Regex;

use std::sync::LazyLock;

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

use ua_generator::{fastrand, ua};

pub fn random_user_agent() -> &'static str {
    static AGENTS: LazyLock<&'static Vec<&'static str>> = LazyLock::new(ua::all_static_agents);
    log::trace!("Spoofing user agent...");
    let ua = AGENTS[fastrand::usize(..AGENTS.len())];
    log::trace!("Spoofed user agent: {ua}");

    ua
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getting_domain() {
        let url_with_scheme =
            "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
        let url_without_scheme =
            "kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";

        assert_eq!("kodikplayer.com", extract_domain(url_with_scheme).unwrap());
        assert_eq!(
            "kodikplayer.com",
            extract_domain(url_without_scheme).unwrap()
        );
    }

    #[test]
    fn random_agent_is_not_always_same() {
        let a1 = random_user_agent();
        let a2 = random_user_agent();
        assert_ne!(a1, a2);
    }
}
