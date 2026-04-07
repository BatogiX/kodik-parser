use base64::{Engine as _, engine::general_purpose};

use crate::{KODIK_STATE, scraper::KodikResponse};
use kodik_utils::KodikError;

const MIN_SHIFT: u8 = 0;
const MAX_SHIFT: u8 = 26;

/// Decodes links in the Kodik response.
///
/// # Errors
///
/// Returns a `KodikError` if decoding fails for any of the links.
pub fn decode_links(kodik_response: &mut KodikResponse) -> Result<(), KodikError> {
    log::debug!("Decoding links...");

    for link in &mut kodik_response.links.quality_360 {
        link.src = decode_link(&link.src)?;
    }

    let base_360 = kodik_response.links.quality_360.first();

    for link in &mut kodik_response.links.quality_480 {
        link.src = match base_360 {
            Some(link) => link.src.replace("/360.mp4", "/480.mp4"),
            None => decode_link(&link.src)?,
        };
    }

    for link in &mut kodik_response.links.quality_720 {
        link.src = match base_360 {
            Some(link) => link.src.replace("/360.mp4", "/720.mp4"),
            None => decode_link(&link.src)?,
        };
    }

    log::trace!("Decoded links: {:#?}", kodik_response.links);
    Ok(())
}

fn decode_link(src: &str) -> Result<String, KodikError> {
    let shift = KODIK_STATE.shift().clamp(MIN_SHIFT, MAX_SHIFT);

    if let Ok(decoded) = try_decode(src, shift) {
        return Ok(decoded);
    }

    for shift in MIN_SHIFT..=MAX_SHIFT {
        if let Ok(decoded) = try_decode(src, shift) {
            KODIK_STATE.set_shift(shift);
            return Ok(decoded);
        }
    }

    Err(KodikError::LinkCannotBeDecoded(src.to_owned()))
}

fn try_decode(encoded: &str, shift: u8) -> Result<String, KodikError> {
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

fn caesar_cipher(text: &str, shift: u8) -> String {
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
pub fn decode_base64(input: &str) -> Result<String, KodikError> {
    let decoded_input = general_purpose::STANDARD.decode(input)?;
    Ok(String::from_utf8(decoded_input)?)
}

#[cfg(test)]
mod tests {
    use crate::scraper::{Link, Links};

    use super::*;

    #[test]
    fn b64_test() {
        let input = "L2Z0b3I=";
        let decoded = decode_base64(input).unwrap();
        assert_eq!("/ftor", decoded);
    }

    #[test]
    fn caesar_cipher_test() {
        let text = "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = caesar_cipher(text, 8);
        assert_eq!(
            "aHR0cHM6Ly9wNTYua29kaWsuaW5mby9zL20vTHk5amJHOTFaQzVyYjJScGF5MXpkRzl5WVdkbExtTnZiUzkxYzJWeWRYQnNiMkZrY3k4ek9Ua3lZbVpoT1MwNVlqYzNMVFE0WlRJdE9HWmpZUzA1WkdSbVlUZzVNelJoT0RVLzE1YjIyNTlkOTk1YzZjNWU1N2Q0NmNmNjYwNTYwNjZhMTE2MmY3MzRiNTBjYTRmYzE5MjZhYTZmMjg0N2MwMTA6MjAyNTA4MTQyMS8zNjAubXA0OmhsczptYW5pZmVzdC5tM3U4",
            decoded
        );
    }

    #[test]
    fn try_decoding() {
        let src = "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = try_decode(src, 8).unwrap();
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/360.mp4:hls:manifest.m3u8",
            decoded
        );
    }

    #[test]
    fn decoding_link() {
        let src = "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = decode_link(src).unwrap();
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/360.mp4:hls:manifest.m3u8",
            decoded
        );
    }

    #[test]
    fn decoding_links() {
        let mut kodik_response = KodikResponse {
    links: Links {
        quality_360: vec![
            Link {
                src: "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                r#type: "application/x-mpegURL".to_owned()
            },
        ],
        quality_480: vec![
            Link {
                src: "iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                r#type: "application/x-mpegURL".to_owned()
            },
        ],
        quality_720: vec![
            Link {
                src: "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                r#type: "application/x-mpegURL".to_owned()
            },
        ],
    },
};
        decode_links(&mut kodik_response).unwrap();

        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/360.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_360[0].src
        );
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/480.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_480[0].src
        );
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/720.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_720[0].src
        );
    }
}
