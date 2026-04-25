use std::fs::OpenOptions;

use kodik_parser::KODIK_STATE;

use crate::{
    cache::{CACHE_PATH, Cache},
    config::Config,
};

fn load_test() -> Cache {
    let mut cache = Cache::load().unwrap();

    if cache.endpoint.as_ref().unwrap().is_empty() || cache.shift.unwrap() == 0 {
        let cache_path = CACHE_PATH.as_ref().unwrap();
        cache = Cache {
            shift: Some(13),
            endpoint: Some(String::from("/abcd")),
            cookie: None,
            path: CACHE_PATH.as_ref().unwrap().to_owned(),
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
    cache.apply(&mut Config::build(vec!["kodik".to_string(), "test".to_string()]).unwrap());
    assert!(!KODIK_STATE.endpoint().is_empty());
    assert_ne!(KODIK_STATE.shift(), 0);
}
