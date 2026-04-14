use kodik_utils::KodikError;

pub fn extract_id(url: &str) -> Result<&str, KodikError> {
    let id_re = lazy_regex::regex!(r"/animes?/(?:[a-z])?([0-9]+)(?:-|$|/)");

    id_re
        .captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .ok_or(KodikError::Regex("id not found in url"))
}
