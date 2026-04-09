use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::LazyLock,
};

use kodik_parser::KODIK_STATE;
use serde::{Deserialize, Serialize};

pub static CACHE_PATH: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| dirs::cache_dir().map(|cache_dir| cache_dir.join("kodik").join("cache.json")));

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cache {
    pub shift: u8,
    pub endpoint: String,
}

impl Cache {
    pub fn load() -> Self {
        let Some(cache_path) = CACHE_PATH.as_ref() else {
            return Self::default();
        };

        if !cache_path.exists() {
            if let Some(parent) = cache_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = File::create(cache_path);
        }

        fs::read_to_string(cache_path).map_or_else(
            |_| Self::default(),
            |content| serde_json::from_str(&content).unwrap_or_default(),
        )
    }

    pub fn save(&self) -> Option<()> {
        let cache_path = CACHE_PATH.as_ref()?;
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(cache_path)
            .ok()?;
        serde_json::to_writer_pretty(file, self).ok()
    }

    pub fn update(&mut self) {
        self.shift = KODIK_STATE.shift();
        self.endpoint.clone_from(&KODIK_STATE.endpoint());
    }

    pub fn is_changed(&self) -> bool {
        self.shift != KODIK_STATE.shift()
            || self.endpoint.as_str() != KODIK_STATE.endpoint().as_str()
    }

    pub fn apply(&self) {
        KODIK_STATE.set_shift(self.shift);
        KODIK_STATE.set_endpoint(self.endpoint.clone());
    }
}
