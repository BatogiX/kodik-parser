use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU8, Ordering},
};

use arc_swap::{ArcSwap, Guard};

pub static KODIK_STATE: LazyLock<KodikState> = LazyLock::new(KodikState::default);

#[derive(Debug, Default)]
pub struct KodikState {
    pub(crate) endpoint: ArcSwap<String>,
    pub(crate) shift: AtomicU8,
}

impl KodikState {
    pub fn load_endpoint(&self) -> Guard<Arc<String>> {
        self.endpoint.load()
    }

    pub fn load_full_endpoint(&self) -> Arc<String> {
        self.endpoint.load_full()
    }

    pub fn store_endpoint(&self, endpoint: Arc<String>) {
        self.endpoint.store(endpoint);
    }

    pub fn load_shift(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn store_shift(&self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }
}
