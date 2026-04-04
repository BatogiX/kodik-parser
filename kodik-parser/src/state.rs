use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU8, Ordering},
};

use arc_swap::{ArcSwap, Guard};
use tokio::sync::Notify;

pub static KODIK_STATE: LazyLock<KodikState> = LazyLock::new(KodikState::default);

#[derive(Debug, Default)]
pub struct KodikState {
    pub(crate) endpoint: ArcSwap<String>,
    pub(crate) shift: AtomicU8,
    pub(crate) notify: Notify,
}

impl KodikState {
    pub fn shift(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn set_shift(&self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }

    pub fn endpoint(&self) -> Guard<Arc<String>> {
        self.endpoint.load()
    }

    pub fn set_endpoint(&self, endpoint: String) {
        self.endpoint.store(Arc::new(endpoint));
    }
}
