use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::LazyLock,
};

use kodik_parser::KODIK_STATE;
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub static CACHE_PATH: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| dirs::cache_dir().map(|cache_dir| cache_dir.join("kodik").join("cache.json")));

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub shift: Option<u8>,
    pub endpoint: Option<String>,
    pub cookie: Option<String>,
    #[serde(skip)]
    pub path: PathBuf,
}

impl Cache {
    pub fn load() -> Option<Self> {
        let cache_path = CACHE_PATH.as_ref()?;

        if !cache_path.exists() {
            if let Some(parent) = cache_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = File::create(cache_path);
        }

        match fs::read_to_string(cache_path) {
            Ok(content) => {
                let mut cache = serde_json::from_str::<Self>(&content).ok()?;
                cache.path.clone_from(&cache_path.to_owned());
                Some(cache)
            }
            Err(_) => None,
        }
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

    pub fn update(&mut self, cookie: Option<&str>) {
        self.shift = Some(KODIK_STATE.shift());
        let endpoint = KODIK_STATE.endpoint().to_string();
        self.endpoint.clone_from(&Some(endpoint));
        self.cookie = cookie.map(ToOwned::to_owned);
    }

    pub fn is_changed(&self, cookie: Option<&str>) -> bool {
        self.shift != Some(KODIK_STATE.shift())
            || self.endpoint.as_deref() != Some(KODIK_STATE.endpoint().as_str())
            || self.cookie.as_deref() != cookie
    }

    pub fn apply(&self, config: &mut Config) {
        if let Some(shift) = self.shift {
            KODIK_STATE.set_shift(shift);
        }
        if let Some(endpoint) = self.endpoint.clone() {
            KODIK_STATE.set_endpoint(endpoint);
        }

        if config.cookie.is_none() && self.cookie.is_some() {
            config.cookie.clone_from(&self.cookie.clone());
        }
    }
}
