use ua_generator::{
    fastrand::{self, Rng},
    ua,
};

#[must_use]
pub fn random_user_agent() -> &'static str {
    log::debug!("Spoofing user agent...");

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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn random_agent_is_not_always_same() {
        let a1 = random_user_agent();
        let a2 = random_user_agent();
        assert_ne!(a1, a2);
    }
}
