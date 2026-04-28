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
    fn decode_src(&mut self) -> Result<(), Error> {
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
