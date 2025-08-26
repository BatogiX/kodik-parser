use std::sync::atomic::{AtomicU8, Ordering};

use base64::{Engine as _, engine::general_purpose};

use crate::scraper::KodikResponse;

static SHIFT: AtomicU8 = AtomicU8::new(0);

pub fn decode_links(kodik_response: &mut KodikResponse) -> Result<(), Box<dyn std::error::Error>> {
    for link_360 in &mut kodik_response.links.quality_360 {
        link_360.src = decode_link(&link_360.src)?;
    }

    for link_480 in &mut kodik_response.links.quality_480 {
        let link_360 = kodik_response.links.quality_360.first();
        match link_360 {
            Some(link_360) => {
                link_480.src = link_360.src.replace("/360.mp4", "/480.mp4");
            }
            None => {
                link_480.src = decode_link(&link_480.src)?;
            }
        }
    }

    for link_720 in &mut kodik_response.links.quality_720 {
        let link_360 = kodik_response.links.quality_360.first();
        match link_360 {
            Some(link_360) => {
                link_720.src = link_360.src.replace("/360.mp4", "/720.mp4");
            }
            None => {
                link_720.src = decode_link(&link_720.src)?;
            }
        }
    }

    Ok(())
}

fn decode_link(src: &str) -> Result<String, Box<dyn std::error::Error>> {
    let shift = SHIFT.load(Ordering::Relaxed);

    if shift != 0
        && let Ok(decoded) = try_decode(src, shift)
    {
        return Ok(decoded);
    }

    for shift in 1..=25 {
        if let Ok(decoded) = try_decode(src, shift) {
            SHIFT.store(shift, Ordering::Relaxed);
            return Ok(decoded);
        }
    }

    Err(format!("Src: {src} cannot be decoded").into())
}

fn try_decode(src: &str, shift: u8) -> Result<String, Box<dyn std::error::Error>> {
    let mut decoded_caesar = caesar_cipher(src, shift);

    while decoded_caesar.len() % 4 != 0 {
        decoded_caesar.push('=');
    }

    b64(&decoded_caesar)
}

fn caesar_cipher(text: &str, shift: u8) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };

                let pos = c as u8 - base;
                let new_pos = (pos + 26 - shift) % 26;
                (base + new_pos) as char
            } else {
                c
            }
        })
        .collect()
}

pub fn b64(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_input = general_purpose::STANDARD.decode(input)?;

    Ok(String::from_utf8(decoded_input)?)
}

#[cfg(test)]
mod tests {
    use crate::scraper::{Link, Links};

    use super::*;

    #[test]
    fn test_b64() {
        let input = "L2Z0b3I=";
        let decoded = b64(input).unwrap();
        assert_eq!("/ftor", decoded);
    }

    #[test]
    fn test_caesar_cipher() {
        let text = "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = caesar_cipher(text, 8);
        assert_eq!(
            "aHR0cHM6Ly9wNTYua29kaWsuaW5mby9zL20vTHk5amJHOTFaQzVyYjJScGF5MXpkRzl5WVdkbExtTnZiUzkxYzJWeWRYQnNiMkZrY3k4ek9Ua3lZbVpoT1MwNVlqYzNMVFE0WlRJdE9HWmpZUzA1WkdSbVlUZzVNelJoT0RVLzE1YjIyNTlkOTk1YzZjNWU1N2Q0NmNmNjYwNTYwNjZhMTE2MmY3MzRiNTBjYTRmYzE5MjZhYTZmMjg0N2MwMTA6MjAyNTA4MTQyMS8zNjAubXA0OmhsczptYW5pZmVzdC5tM3U4",
            decoded
        );
    }

    #[test]
    fn test_try_decode() {
        let src = "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = try_decode(src, 8).unwrap();
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/360.mp4:hls:manifest.m3u8",
            decoded
        );
    }

    #[test]
    fn test_decode_link() {
        let src = "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4";
        let decoded = decode_link(src).unwrap();
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/360.mp4:hls:manifest.m3u8",
            decoded
        );
    }

    #[test]
    fn test_decode_links() {
        let mut kodik_response = KodikResponse {
    links: Links {
        quality_360: vec![
            Link {
                src: "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                r#type: "application/x-mpegURL".to_owned(),
            },
        ],
        quality_480: vec![
            Link {
                src: "iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                r#type: "application/x-mpegURL".to_owned(),
            },
        ],
        quality_720: vec![
            Link {
                src: "iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4".to_owned(),
                r#type: "application/x-mpegURL".to_owned(),
            },
        ],
    },
};
        decode_links(&mut kodik_response).unwrap();

        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/360.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_360.first().unwrap().src
        );
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/480.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_480.first().unwrap().src
        );
        assert_eq!(
            "https://p56.kodik.info/s/m/Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/15b2259d995c6c5e57d46cf66056066a1162f734b50ca4fc1926aa6f2847c010:2025081421/720.mp4:hls:manifest.m3u8",
            kodik_response.links.quality_720.first().unwrap().src
        );
    }
}
