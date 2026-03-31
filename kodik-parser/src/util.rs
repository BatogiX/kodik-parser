use std::sync::LazyLock;

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
    fn random_agent_is_not_always_same() {
        let a1 = random_user_agent();
        let a2 = random_user_agent();
        assert_ne!(a1, a2);
    }
}
