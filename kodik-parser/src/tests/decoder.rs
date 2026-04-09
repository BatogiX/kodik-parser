use crate::{
    Link, Links, Response,
    decoder::{caesar_cipher, decode_base64, decode_link, decode_links, try_decode},
};

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
    let mut kodik_response = Response {
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
