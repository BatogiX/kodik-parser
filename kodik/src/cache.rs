use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use directories::BaseDirs;
use kodik_parser::cache::KODIK_CACHE;
use serde::{Deserialize, Serialize};

static CACHE_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    BaseDirs::new().map(|base_dirs| base_dirs.cache_dir().join("kodik").join("cache.json"))
});

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cache {
    pub shift: u8,
    pub video_info_endpoint: Arc<String>,
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

    fn save(&self) -> Option<()> {
        let cache_path = CACHE_PATH.as_ref()?;
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(cache_path)
            .ok()?;
        serde_json::to_writer_pretty(file, self).ok()
    }

    pub fn update(&mut self) {
        self.shift = KODIK_CACHE.shift_load();
        self.video_info_endpoint
            .clone_from(&KODIK_CACHE.endpoint_load());
        self.save();
    }

    pub fn is_changed(&self) -> bool {
        self.shift != KODIK_CACHE.shift_load()
            || self.video_info_endpoint.as_str() != KODIK_CACHE.endpoint_load().as_str()
    }

    pub fn apply_to_globals(&self) {
        KODIK_CACHE.shift_store(self.shift);
        KODIK_CACHE.endpoint_store(Arc::clone(&self.video_info_endpoint));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_test() {
        if Cache::load().is_none() {
            let cache_path = CACHE_PATH.as_ref().unwrap();
            let cache = Cache {
                shift: 13,
                video_info_endpoint: Arc::from("/abcd".to_owned()),
            };
            let file = OpenOptions::new().write(true).open(cache_path).unwrap();
            serde_json::to_writer_pretty(file, &cache).unwrap();
            Cache::load().unwrap();
        }
    }
}
