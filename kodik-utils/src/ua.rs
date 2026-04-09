use ua_generator::{
    fastrand::{self, Rng},
    ua,
};

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
