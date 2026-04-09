use std::sync::LazyLock;

use ua_generator::{fastrand, ua};

pub fn random_user_agent() -> &'static str {
    static AGENTS: LazyLock<&'static Vec<&'static str>> = LazyLock::new(ua::all_static_agents);
    log::trace!("Spoofing user agent...");
    let ua = AGENTS[fastrand::usize(..AGENTS.len())];
    log::trace!("Spoofed user agent: {ua}");

    ua
}
