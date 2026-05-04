use crate::{KODIK_STATE, Link, scraper::Response};
use base64::{Engine as _, engine::general_purpose};
use kodik_utils::Error;

const MIN_SHIFT: u8 = 0;
const MAX_SHIFT: u8 = 26;

impl Response {
    /// Decodes links in the Kodik response.
    ///
    /// # Errors
    ///
    /// Returns a `KodikError` if decoding fails for any of the links.
    pub(crate) fn decode_links(&mut self) -> Result<(), Error> {
        log::debug!("Decoding links...");

        for link in &mut self.links.quality_360 {
            link.decode_src()?;
        }

        for link in &mut self.links.quality_480 {
            link.decode_src()?;
        }

        for link in &mut self.links.quality_720 {
            link.decode_src()?;
        }

        log::trace!("Decoded links: {:#?}", self.links);
        Ok(())
    }
}

impl Link {
    pub(crate) fn decode_src(&mut self) -> Result<(), Error> {
        let shift = KODIK_STATE.shift().clamp(MIN_SHIFT, MAX_SHIFT);

        if let Ok(decoded) = try_decode(&self.src, shift) {
            self.src = decoded;
            return Ok(());
        }

        for shift in MIN_SHIFT..=MAX_SHIFT {
            if let Ok(decoded) = try_decode(&self.src, shift) {
                KODIK_STATE.set_shift(shift);
                self.src = decoded;
                return Ok(());
            }
        }

        Err(Error::LinkCannotBeDecoded(self.src.clone()))
    }
}

pub fn try_decode(encoded: &str, shift: u8) -> Result<String, Error> {
    let mut decoded_caesar = caesar_cipher(encoded, shift);

    while !decoded_caesar.len().is_multiple_of(4) {
        decoded_caesar.push('=');
    }

    let decode_result = decode_base64(&decoded_caesar);

    if let Ok(mut decoded) = decode_result {
        if !decoded.starts_with("https:") {
            decoded.insert_str(0, "https:");
        }
        return Ok(decoded);
    }

    decode_result
}

pub fn caesar_cipher(text: &str, shift: u8) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                let pos = c as u8 - base;
                let new_pos = (pos + MAX_SHIFT - shift) % MAX_SHIFT;
                (base + new_pos) as char
            } else {
                c
            }
        })
        .collect()
}

/// Decodes a base64-encoded string.
///
/// # Errors
///
/// Returns a `KodikError` if decoding fails due to invalid base64 input or invalid UTF-8.
pub fn decode_base64(input: &str) -> Result<String, Error> {
    let decoded_input = general_purpose::STANDARD.decode(input)?;
    Ok(String::from_utf8(decoded_input)?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use crate::Links;

    use super::*;

    #[test]
    fn b64_test() {
        let input = "L2Z0b3I=";
        let decoded = decode_base64(input).unwrap();
        assert_eq!("/ftor", decoded);
    }

    #[test]
    fn caesar_cipher_test() {
        let text = "iPZ0kPU6Tg9eUBUck29aj2ZrHO4cG29bT3UdjA9pANQeG0pVVsf5WExqZhsfEsU1muQgmPHiZ05zGus1iuQgUPHsEM5aG25El2RPWEpiAM12EDlRU01FAFlHist0EsZHUNxxULJVD0shBNlRms9MH3ZVms0fBCZrUNtyBBRVms13T2MhVrMhHLJrU2C4GhpuGuYgVLG4UrCgHLHqGrprGuU2VhtuHLUhULQhUhU2VuZuVOC1VONsHEM2HuC0WEU1WLG6UrIgVrI1ULMeUq8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = caesar_cipher(text, 8);
        assert_eq!(
            "aHR0cHM6Ly9wMTMuc29sb2RjZG4uY29tL3MvbS9hSFIwY0hNNkx5OWpiRzkxWkM1emIyeHZaR05rYmk1amIyMHZkWE5sY25Wd2JHOWhaSE12WVdJM01XSXdZakl0WkRZMFppMDBNV0kzTFdJek9EZ3RNek0xTURjMFlqTTJNek13L2EzNjEzZDBjM2U4YzhmYmQyNDY4MjUyZDZiYjhjYmM2NzlmZDMzMDIzMzM2NmRmNGU1NGFkZWE2ZmU0OWM1ODY6MjAyNjA1MDEwMi8zNjAubXA0OmhsczptYW5pZmVzdC5tM3U4",
            decoded
        );
    }

    #[test]
    fn try_decoding() {
        let src = "iPZ0kPU6Tg9eUBUck29aj2ZrHO4cG29bT3UdjA9pANQeG0pVVsf5WExqZhsfEsU1muQgmPHiZ05zGus1iuQgUPHsEM5aG25El2RPWEpiAM12EDlRU01FAFlHist0EsZHUNxxULJVD0shBNlRms9MH3ZVms0fBCZrUNtyBBRVms13T2MhVrMhHLJrU2C4GhpuGuYgVLG4UrCgHLHqGrprGuU2VhtuHLUhULQhUhU2VuZuVOC1VONsHEM2HuC0WEU1WLG6UrIgVrI1ULMeUq8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = try_decode(src, 8).unwrap();
        assert_eq!(
            "https://p13.solodcdn.com/s/m/aHR0cHM6Ly9jbG91ZC5zb2xvZGNkbi5jb20vdXNlcnVwbG9hZHMvYWI3MWIwYjItZDY0Zi00MWI3LWIzODgtMzM1MDc0YjM2MzMw/a3613d0c3e8c8fbd2468252d6bb8cbc679fd330233366df4e54adea6fe49c586:2026050102/360.mp4:hls:manifest.m3u8",
            decoded
        );
    }

    #[test]
    fn decoding_link() {
        let mut link = Link { src: "iPZ0kPU6Tg9eUBUck29aj2ZrHO4cG29bT3UdjA9pANQeG0pVVsf5WExqZhsfEsU1muQgmPHiZ05zGus1iuQgUPHsEM5aG25El2RPWEpiAM12EDlRU01FAFlHist0EsZHUNxxULJVD0shBNlRms9MH3ZVms0fBCZrUNtyBBRVms13T2MhVrMhHLJrU2C4GhpuGuYgVLG4UrCgHLHqGrprGuU2VhtuHLUhULQhUhU2VuZuVOC1VONsHEM2HuC0WEU1WLG6UrIgVrI1ULMeUq8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(), r#type: "application/x-mpegURL".to_owned() };
        link.decode_src().unwrap();
        assert_eq!(
            "https://p13.solodcdn.com/s/m/aHR0cHM6Ly9jbG91ZC5zb2xvZGNkbi5jb20vdXNlcnVwbG9hZHMvYWI3MWIwYjItZDY0Zi00MWI3LWIzODgtMzM1MDc0YjM2MzMw/a3613d0c3e8c8fbd2468252d6bb8cbc679fd330233366df4e54adea6fe49c586:2026050102/360.mp4:hls:manifest.m3u8",
            link.src
        );
    }

    #[test]
    fn decoding_links() {
        let mut kodik_response = Response {
        links: Links {
            quality_360: vec![
                Link {
                    src: "iPZ0kPU6Tg9eUBUck29aj2ZrHO4cG29bT3UdjA9pANQeG0pVVsf5WExqZhsfEsU1muQgmPHiZ05zGus1iuQgUPHsEM5aG25El2RPWEpiAM12EDlRU01FAFlHist0EsZHUNxxULJVD0shBNlRms9MH3ZVms0fBCZrUNtyBBRVms13T2MhVrMhHLJrU2C4GhpuGuYgVLG4UrCgHLHqGrprGuU2VhtuHLUhULQhUhU2VuZuVOC1VONsHEM2HuC0WEU1WLG6UrIgVrI1ULMeUq8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                    r#type: "application/x-mpegURL".to_owned()
                },
            ],
            quality_480: vec![
                Link {
                    src: "iPZ0kPU6Tg9eUBUck29aj2ZrHO4cG29bT3UdjA9pANQeG0pVVsf5WExqZhsfEsU1muQgmPHiZ05zGus1iuQgUPHsEM5aG25El2RPWEpiAM12EDlRU01FAFlHist0EsZHUNxxULJVD0shBNlRms9MH3ZVms0fBCZrUNtyBBRVms13T2MhVrMhHLJrU2C4GhpuGuYgVLG4UrCgHLHqGrprGuU2VhtuHLUhULQhUhU2VuZuVOC1VONsHEM2HuC0WEU1WLG6UrIgVrI1ULMeUq80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                    r#type: "application/x-mpegURL".to_owned()
                },
            ],
            quality_720: vec![
                Link {
                    src: "iPZ0kPU6Tg9eUBQck29aj2ZrHO4cG29bT3UdjA9pANQeG0pVVsf5WExqZhsfEsU1muQgmPHiZ05zGus1iuQgUPHsEM5aG25El2RPWEpiAM12EDlRU01FAFlHist0EsZHUNxxULJVD0shBNlRms9MH3ZVms0fBCZrUNtyBBRVms13T2MhVrMhHLJrU2C4GhpuGuYgVLG4UrCgHLHqGrprGuU2VhtuHLUhULQhUhU2VuZuVOC1VONsHEM2HuC0WEU1WLG6UrIgVrI1ULMeUq83UrIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                    r#type: "application/x-mpegURL".to_owned()
                },
            ],
        },
    };

        kodik_response.decode_links().unwrap();

        assert_eq!(
            "https://p13.solodcdn.com/s/m/aHR0cHM6Ly9jbG91ZC5zb2xvZGNkbi5jb20vdXNlcnVwbG9hZHMvYWI3MWIwYjItZDY0Zi00MWI3LWIzODgtMzM1MDc0YjM2MzMw/a3613d0c3e8c8fbd2468252d6bb8cbc679fd330233366df4e54adea6fe49c586:2026050102/360.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_360[0].src
        );
        assert_eq!(
            "https://p13.solodcdn.com/s/m/aHR0cHM6Ly9jbG91ZC5zb2xvZGNkbi5jb20vdXNlcnVwbG9hZHMvYWI3MWIwYjItZDY0Zi00MWI3LWIzODgtMzM1MDc0YjM2MzMw/a3613d0c3e8c8fbd2468252d6bb8cbc679fd330233366df4e54adea6fe49c586:2026050102/480.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_480[0].src
        );
        assert_eq!(
            "https://p12.solodcdn.com/s/m/aHR0cHM6Ly9jbG91ZC5zb2xvZGNkbi5jb20vdXNlcnVwbG9hZHMvYWI3MWIwYjItZDY0Zi00MWI3LWIzODgtMzM1MDc0YjM2MzMw/a3613d0c3e8c8fbd2468252d6bb8cbc679fd330233366df4e54adea6fe49c586:2026050102/720.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_720[0].src
        );
    }
}
