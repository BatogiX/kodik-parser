use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::{self, LazyLock, atomic::Ordering},
};

use directories::BaseDirs;
use kodik_parser::{SHIFT, VIDEO_INFO_ENDPOINT};
use serde::{Deserialize, Serialize};

static CACHE_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    BaseDirs::new().map(|base_dirs| base_dirs.cache_dir().join("kodik").join("cache.json"))
});

#[derive(Debug, Serialize, Deserialize)]
pub struct KodikCache {
    pub shift: u8,
    pub video_info_endpoint: String,
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

    fn update(&mut self) {
        self.shift = SHIFT.load(Ordering::Relaxed);
        self.video_info_endpoint.clone_from(
            &VIDEO_INFO_ENDPOINT
                .read()
                .unwrap_or_else(sync::PoisonError::into_inner),
        );
        self.save();
    }

    fn is_changed(&self) -> bool {
        self.shift != SHIFT.load(Ordering::Relaxed)
            || self.video_info_endpoint
                != VIDEO_INFO_ENDPOINT
                    .read()
                    .unwrap_or_else(sync::PoisonError::into_inner)
                    .as_str()
    }

    pub fn apply_to_globals(&self) {
        SHIFT.store(self.shift, Ordering::Relaxed);
        let endpoint_clone = self.video_info_endpoint.clone();
        *VIDEO_INFO_ENDPOINT
            .write()
            .unwrap_or_else(sync::PoisonError::into_inner) = endpoint_clone;
    }

    pub fn persist_if_dirty(&mut self) {
        if self.is_changed() {
            self.update();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use super::*;

    #[test]
    fn load_test() {
        if KodikCache::load().is_none() {
            let cache_path = CACHE_PATH.as_ref().unwrap();
            let cache = KodikCache {
                shift: 13,
                video_info_endpoint: "/abcd".to_owned(),
            };
            let file = OpenOptions::new().write(true).open(cache_path).unwrap();
            serde_json::to_writer_pretty(file, &cache).unwrap();
            KodikCache::load().unwrap();
        }
    }
}
