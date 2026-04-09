use reqwest::Client;

use crate::{
    parse,
    parser::{VideoInfo, extract_endpoint, extract_player_url},
};

#[test]
fn v_info_from_response_test() {
    let expected_video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

    let html = "
  var videoInfo = {};
   vInfo.type = 'video';
   vInfo.hash = '060cab655974d46835b3f4405807acc2';
   vInfo.id = '91873';
</script>";

    let video_info = VideoInfo::from_response(html).unwrap();

    assert_eq!(expected_video_info, video_info);
}

#[test]
fn v_info_from_url_test() {
    let expected_video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

    let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2";
    let video_info = VideoInfo::from_url(url).unwrap();

    assert_eq!(expected_video_info, video_info);
}

#[test]
fn getting_player_url() {
    let domain = "kodikplayer.com";
    let html = r#"
  </script>

  <link rel="stylesheet" href="/assets/css/app.player.ffc43caed0b4bc0a9f41f95c06cd8230d49aaf7188dbba5f0770513420541101.css">
  <script type="text/javascript" src="/assets/js/app.player_single.0a909e421830a88800354716d562e21654500844d220805110c7cf2092d70b05.js"></script>
</head>
<body class=" ">
  <div class="main-box">
    <style>
  .resume-button { color: rgba(255, 255, 255, 0.75); }
  .resume-button:hover { background-color: #171717; }
  .resume-button { border-radius: 3px; }
  .active-player .resume-button { border-radius: 3px; }"#;

    let player_url = extract_player_url(domain, html).unwrap();
    assert_eq!(
        "https://kodikplayer.com/assets/js/app.player_single.0a909e421830a88800354716d562e21654500844d220805110c7cf2092d70b05.js",
        player_url
    );
}

#[test]
fn getting_endpoint() {
    let player_html = r#"==t.secret&&(e.secret=t.secret),userInfo&&"object"===_typeof(userInfo.info)&&(e.info=JSON.stringify(userInfo.info)),void 0!==window.advertTest&&(e.a_test=!0),!0===t.isUpdate&&(e.isUpdate=!0),$.ajax({type:"POST",url:atob("L2Z0b3I="),"#;
    assert_eq!("/ftor", extract_endpoint(player_html).unwrap());
}

#[test]
fn video_info_serializing() {
    let video_info = VideoInfo::new("video", "060cab655974d46835b3f4405807acc2", "91873");

    let serialized = serde_json::to_string(&video_info).unwrap();
    assert_eq!(
        r#"{"type":"video","hash":"060cab655974d46835b3f4405807acc2","id":"91873","bad_user":"True","info":"{}","cdn_is_working":"True"}"#,
        serialized
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn async_parse() {
    let client = Client::new();
    let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let kodik_response = parse(&client, url).await.unwrap();
    println!("{kodik_response:#?}");
}
