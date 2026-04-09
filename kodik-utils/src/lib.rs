pub mod error;
pub use error::KodikError;

use lazy_regex::{self, Lazy};
use regex_lite::Regex;
use ua_generator::{
    fastrand::{self, Rng},
    ua,
};

/// Extracts the domain from a URL.
///
/// # Errors
///
/// Returns `KodikError::Regex` if no valid domain is found in the URL.
pub fn extract_domain(url: &str) -> Result<&str, KodikError> {
    let domain_re: &Lazy<Regex> = lazy_regex::regex!(
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

#[must_use]
pub fn random_user_agent() -> &'static str {
    log::trace!("Spoofing user agent...");

    let agents = ua::all_static_agents();
    let index = fastrand::usize(..agents.len());
    let ua = agents
        .get(index)
        .copied()
        .unwrap_or_else(|| ua::spoof_random_agent(&mut Rng::new()));

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
