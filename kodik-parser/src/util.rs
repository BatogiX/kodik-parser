use std::sync::{Arc, LazyLock};

use tokio::sync::RwLockWriteGuard;
use ua_generator::{fastrand, ua};

use reqwest::Client;

use crate::{
    KodikError,
    parser::{extract_endpoint, extract_player_url},
    scraper::get,
};

pub(crate) async fn update_endpoint(
    client: &Client,
    domain: &str,
    html: &str,
    mut endpoint: RwLockWriteGuard<'_, Arc<str>>,
) -> Result<(), KodikError> {
    let player_url = extract_player_url(domain, html)?;
    let player_html = get(client, &player_url).await?;
    let new_endpoint = extract_endpoint(&player_html)?;
    *endpoint = Arc::from(new_endpoint);

    Ok(())
}

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
    fn random_agent_is_not_always_same() {
        let a1 = random_user_agent();
        let a2 = random_user_agent();
        assert_ne!(a1, a2);
    }
}
