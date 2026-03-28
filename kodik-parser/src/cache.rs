use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU8, Ordering},
};

use arc_swap::{ArcSwap, Guard};

pub static KODIK_CACHE: LazyLock<KodikCache> = LazyLock::new(KodikCache::default);

#[derive(Debug, Default)]
pub struct KodikCache {
    pub(crate) endpoint: ArcSwap<String>,
    pub(crate) shift: AtomicU8,
}

impl KodikCache {
    pub fn endpoint_load(&self) -> Guard<Arc<String>> {
        self.endpoint.load()
    }

    pub fn endpoint_store(&self, endpoint: Arc<String>) {
        self.endpoint.store(endpoint);
    }

    pub fn shift_load(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn shift_store(&self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }

    pub fn endpoint(&self) -> &ArcSwap<String> {
        &self.endpoint
    }

    pub fn shift(&self) -> &AtomicU8 {
        &self.shift
    }
}
