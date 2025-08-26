use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// Response structure for player data containing video links
pub struct KodikResponse {
    /// Available video links organized by quality
    pub links: Links,
}

#[derive(Debug, Deserialize)]
/// Container for video links organized by different quality levels
pub struct Links {
    /// Video links for 360p quality
    #[serde(rename = "360")]
    pub quality_360: Vec<Link>,
    /// Video links for 480p quality
    #[serde(rename = "480")]
    pub quality_480: Vec<Link>,
    /// Video links for 720p quality
    #[serde(rename = "720")]
    pub quality_720: Vec<Link>,
}

#[derive(Debug, Deserialize)]
/// Individual video link with source URL and content type
pub struct Link {
    /// Source URL of the video stream
    pub src: String,
    /// MIME type of the video content
    pub r#type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_deserialization() {
        let json = r#"{
            "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
            "type":"application/x-mpegURL"
        }"#;

        let _: Link = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_links_deserialization() {
        let json = r#"{
            "360":[
                {
                    "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                    "type":"application/x-mpegURL"
                }
            ],
            "480":[
                {
                    "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                    "type":"application/x-mpegURL"
                }
            ],
            "720":[
                {
                    "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg83UrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                    "type":"application/x-mpegURL"
                }
            ]
        }"#;

        let _: Links = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_kodik_response_deserialization() {
        let json = r#"{
            "links":{
                "360":[
                    {
                        "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                        "type":"application/x-mpegURL"
                    }
                ],
                "480":[
                    {
                        "src":"iPZ0kPU6Tg9eUhYci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg80WLIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                        "type":"application/x-mpegURL"
                    }
                ],
                "720":[
                    {
                        "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDT2Q3VBJpVrU4UBC3HrprU2RpWEVuVhlrHBs3UhHsVBI5UORuVBVpGrptHBlqV2QgVLpuGuY2GhHqUOVtVBG0WLs6UrIgVBI4UrGeUg83UrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
                        "type":"application/x-mpegURL"
                    }
                ]
            }
        }"#;

        let _: KodikResponse = serde_json::from_str(json).unwrap();
    }
}
