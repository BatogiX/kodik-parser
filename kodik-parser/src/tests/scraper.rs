use crate::{Link, Links, Response};

#[test]
fn link_deserialization() {
    let json = r#"{
            "src":"iPZ0kPU6Tg9eVBGci29siEaciE5ujg9hT20dBPs5iuRPWBNiYhDgGrRAkON5UFxsZht5EDlsjMfbBvHqChsfGhREmEZGYvVqUsHzG3s4ms9Ci3tHjDxwB1UeVDtyGhVUDNM0EtZRlM9PEuxHChI1EslAjDtCHhDVmtRwB0ZDThM1GrQgVBtsWBs1GhHrVEC1V2Y0VuVuVrGeVBGeVrHpUBM2UuG3UhZqVBJrGBZuGhM5UrHpGBHuUro0V2UeUBI6UrIgVBI4UBYgUA8hVrIcjFI0WupakhxbGE5xHuDhlK5bU3C4",
            "type":"application/x-mpegURL"
        }"#;

    let _: Link = serde_json::from_str(json).unwrap();
}

#[test]
fn links_deserialization() {
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
fn kodik_response_deserialization() {
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

    let _: Response = serde_json::from_str(json).unwrap();
}
