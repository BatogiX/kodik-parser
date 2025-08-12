use std::sync::LazyLock;

use ua_generator::{fastrand, ua};

pub fn spoof_random_agent() -> &'static str {
    static AGENTS: LazyLock<&'static Vec<&'static str>> = LazyLock::new(ua::all_static_agents);
    AGENTS[fastrand::usize(..AGENTS.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_agent_is_not_always_same() {
        let a1 = spoof_random_agent();
        let a2 = spoof_random_agent();
        assert_ne!(a1, a2);
    }
}
