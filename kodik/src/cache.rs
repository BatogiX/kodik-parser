use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::LazyLock,
};

use kodik_parser::state::KODIK_STATE;
use serde::{Deserialize, Serialize};

static CACHE_PATH: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| dirs::cache_dir().map(|cache_dir| cache_dir.join("kodik").join("cache.json")));

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cache {
    pub shift: u8,
    pub endpoint: String,
}

impl Cache {
    pub fn load() -> Option<Self> {
        let cache_path = CACHE_PATH.as_ref()?;

        if !fs::exists(cache_path).ok()? {
            fs::create_dir_all(cache_path.parent()?).ok()?;
            File::create(cache_path).ok();
            return None;
        }

        serde_json::from_str(&fs::read_to_string(cache_path).ok()?)
            .unwrap_or_else(|_| Some(Self::default()))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test() -> Cache {
        let cache = Cache::load().unwrap();

        if cache.endpoint.is_empty() {
            let cache_path = CACHE_PATH.as_ref().unwrap();
            let cache = Cache {
                shift: 13,
                endpoint: String::from("/abcd"),
            };
            let file = OpenOptions::new().write(true).open(cache_path).unwrap();
            serde_json::to_writer_pretty(file, &cache).unwrap();
        }

        cache
    }

    #[tokio::test]
    async fn apply_test() {
        let cache = load_test();
        assert!(KODIK_STATE.endpoint().is_empty());
        assert_eq!(KODIK_STATE.shift(), 0);
        cache.apply();
        assert!(!KODIK_STATE.endpoint().is_empty());
        assert_ne!(KODIK_STATE.shift(), 0);
    }
}
