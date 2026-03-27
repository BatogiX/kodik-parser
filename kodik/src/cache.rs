use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use directories::BaseDirs;
use kodik_parser::util::{get_endpoint, get_shift, set_endpoint, set_shift};
use serde::{Deserialize, Serialize};

static CACHE_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    BaseDirs::new().map(|base_dirs| base_dirs.cache_dir().join("kodik").join("cache.json"))
});

#[derive(Debug, Serialize, Deserialize)]
pub struct KodikCache {
    pub shift: u8,
    pub video_info_endpoint: Arc<String>,
}

impl KodikCache {
    pub fn load() -> Option<Self> {
        let cache_path = CACHE_PATH.as_ref()?;

        if !fs::exists(cache_path).ok()? || fs::metadata(cache_path).ok()?.len() == 0 {
            fs::create_dir_all(cache_path.parent()?).ok()?;
            File::create(cache_path).ok();
            return None;
        }

        serde_json::from_str(&fs::read_to_string(cache_path).ok()?).ok()?
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
        self.shift = get_shift();
        self.video_info_endpoint.clone_from(&get_endpoint());
        self.save();
    }

    pub fn is_changed(&self) -> bool {
        self.shift != get_shift() || self.video_info_endpoint.as_str() != get_endpoint().as_str()
    }

    pub fn apply_to_globals(&self) {
        set_shift(self.shift);
        set_endpoint(Arc::clone(&self.video_info_endpoint));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_test() {
        if KodikCache::load().is_none() {
            let cache_path = CACHE_PATH.as_ref().unwrap();
            let cache = KodikCache {
                shift: 13,
                video_info_endpoint: Arc::from("/abcd".to_owned()),
            };
            let file = OpenOptions::new().write(true).open(cache_path).unwrap();
            serde_json::to_writer_pretty(file, &cache).unwrap();
            KodikCache::load().unwrap();
        }
    }
}
