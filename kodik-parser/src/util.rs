use std::sync::{Arc, LazyLock, atomic::Ordering};

use ua_generator::{fastrand, ua};

use crate::{SHIFT, VIDEO_INFO_ENDPOINT};

pub fn spoof_random_ua() -> &'static str {
    static AGENTS: LazyLock<&'static Vec<&'static str>> = LazyLock::new(ua::all_static_agents);
    log::debug!("Spoofing random user agent...");
    let ua = AGENTS[fastrand::usize(..AGENTS.len())];
    log::trace!("Spoofed user agent: {ua}");

    ua
}

pub fn get_endpoint() -> Arc<String> {
    VIDEO_INFO_ENDPOINT.load_full()
}

pub fn set_endpoint(endpoint: Arc<String>) {
    VIDEO_INFO_ENDPOINT.store(endpoint);
}

pub fn get_shift() -> u8 {
    SHIFT.load(Ordering::Relaxed)
}

pub fn set_shift(shift: u8) {
    SHIFT.store(shift, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_agent_is_not_always_same() {
        let a1 = spoof_random_ua();
        let a2 = spoof_random_ua();
        assert_ne!(a1, a2);
    }
}
