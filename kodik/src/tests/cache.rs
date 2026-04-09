use std::fs::OpenOptions;

use kodik_parser::KODIK_STATE;

use crate::cache::{CACHE_PATH, Cache};

fn load_test() -> Cache {
    let mut cache = Cache::load();

    if cache.endpoint.is_empty() || cache.shift == 0 {
        let cache_path = CACHE_PATH.as_ref().unwrap();
        cache = Cache {
            shift: 13,
            endpoint: String::from("/abcd"),
        };
        let file = OpenOptions::new().write(true).open(cache_path).unwrap();
        serde_json::to_writer_pretty(file, &cache).unwrap();
    }

    cache
}

#[test]
fn apply_test() {
    let cache = load_test();
    assert!(KODIK_STATE.endpoint().is_empty());
    assert_eq!(KODIK_STATE.shift(), 0);
    cache.apply();
    assert!(!KODIK_STATE.endpoint().is_empty());
    assert_ne!(KODIK_STATE.shift(), 0);
}
