use std::sync::{
    LazyLock,
    atomic::{AtomicU8, Ordering},
};

use tokio::sync::{RwLock, RwLockReadGuard};

pub static KODIK_STATE: LazyLock<KodikState> = LazyLock::new(KodikState::default);

#[derive(Debug, Default)]
pub struct KodikState {
    pub(crate) endpoint: RwLock<String>,
    pub(crate) shift: AtomicU8,
}

impl KodikState {
    pub fn shift(&self) -> u8 {
        self.shift.load(Ordering::Relaxed)
    }

    pub fn set_shift(&self, shift: u8) {
        self.shift.store(shift, Ordering::Relaxed);
    }

    pub async fn endpoint(&self) -> RwLockReadGuard<'_, String> {
        self.endpoint.read().await
    }

    pub async fn set_endpoint(&self, endpoint: &String) {
        self.endpoint.write().await.clone_from(endpoint);
    }
}
