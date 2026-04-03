use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU8, Ordering},
};

use tokio::sync::RwLock;

pub static KODIK_STATE: LazyLock<KodikState> = LazyLock::new(KodikState::default);

#[derive(Debug, Default)]
pub struct KodikState {
    pub(crate) endpoint: RwLock<Arc<str>>,
    pub(crate) shift: AtomicU8,
}

impl KodikState {
    pub fn shift(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn set_shift(&self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }

    pub async fn endpoint(&self) -> Arc<str> {
        self.endpoint.read().await.clone()
    }

    pub async fn set_endpoint(&self, endpoint: String) {
        *self.endpoint.write().await = Arc::from(endpoint);
    }
}
