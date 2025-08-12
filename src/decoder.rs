use base64::{Engine as _, engine::general_purpose};

use crate::scraper::PlayerResponse;

pub fn decode_links(player_response: &mut PlayerResponse) -> Result<(), Box<dyn std::error::Error>> {
    for link in &mut player_response.links.quality_360 {
        link.src = decode_link(&link.src)?;
    }

    for link in &mut player_response.links.quality_480 {
        link.src = decode_link(&link.src)?;
    }

    for link in &mut player_response.links.quality_720 {
        link.src = decode_link(&link.src)?;
    }

    Ok(())
}

fn decode_link(src: &str) -> Result<String, Box<dyn std::error::Error>> {
    for shift in 1..26 {
        let mut decoded_caesar = caesar_cipher(src, shift);

        if decoded_caesar.len() % 4 != 0 {
            decoded_caesar.push('=');
        }

        if let Ok(decoded_link) = b64(&decoded_caesar) {
            return Ok(decoded_link);
        }
    }

    Err(format!("Src: {src} cannot be decoded").into())
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
