use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use arc_swap::ArcSwap;
use tokio::sync::Notify;

pub static KODIK_STATE: LazyLock<KodikState> = LazyLock::new(KodikState::default);

#[derive(Debug, Default)]
pub struct KodikState {
    endpoint: ArcSwap<String>,
    shift: AtomicU8,
    notify: Notify,
    updating: AtomicBool,
}

impl KodikState {
    pub fn shift(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn set_shift(&self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }

    pub fn endpoint(&self) -> Arc<String> {
        self.endpoint.load_full()
    }

    pub fn set_endpoint(&self, endpoint: String) {
        self.endpoint.store(Arc::new(endpoint));
    }

    pub(crate) fn clear_endpoint(&self) {
        self.set_endpoint(String::new());
    }

    pub(crate) fn try_begin_update(&self) -> bool {
        !self.updating.swap(true, Ordering::AcqRel)
    }

    pub(crate) fn finish_update(&self, endpoint: String) {
        self.set_endpoint(endpoint);
        self.updating.store(false, Ordering::Release);
        self.notify.notify_waiters();
    }

    pub(crate) async fn wait_for_update(&self) {
        self.notify.notified().await;
    }
}
