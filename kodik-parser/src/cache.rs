use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU8, Ordering},
};

use arc_swap::ArcSwap;

pub static KODIK_CACHE: LazyLock<KodikCache> = LazyLock::new(KodikCache::default);

#[derive(Debug, Default)]
pub struct KodikCache {
    endpoint: ArcSwap<String>,
    shift: AtomicU8,
}

impl KodikCache {
    pub fn get_endpoint(&self) -> Arc<String> {
        self.endpoint.load_full()
    }

    pub fn set_endpoint(&mut self, endpoint: Arc<String>) {
        self.endpoint.store(endpoint);
    }

    pub fn get_shift(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn set_shift(&mut self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }
}
