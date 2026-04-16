use std::env;

use reqwest::Client;

use crate::scraper::{
    SearchResult, Translation, TranslationType, find_search_result, get_kodik_videos,
    get_user_rate, run,
};

#[tokio::test]
async fn get_user_rate_test() {
    let client = Client::new();
    let domain = "shikimori.io";
    let id = "43";
    let cookie = env::var("_kawai_session").unwrap();

    println!(
        "{:#?}",
        get_user_rate(&client, domain, id, &cookie)
            .await
            .unwrap()
            .unwrap()
    );
}

#[tokio::test]
async fn get_kodik_videos_season_test() {
    let client = Client::new();
    let id = "467";

    println!("{:#?}", get_kodik_videos(&client, id).await.unwrap());
}

#[tokio::test]
async fn get_kodik_videos_film_test() {
    let client = Client::new();
    let id = "43";

    println!("{:#?}", get_kodik_videos(&client, id).await.unwrap());
}

#[tokio::test]
async fn run_season_test() {
    let client = Client::new();
    let url = "https://shikimori.io/animes/467-koukaku-kidoutai-stand-alone-complex";
    println!(
        "{:#?}",
        run(&client, url, None, None, None, None).await.unwrap()
    );
}

#[tokio::test]
async fn run_film_test() {
    let client = Client::new();
    let url = "https://shikimori.io/animes/43-koukaku-kidoutai";

    println!(
        "{:#?}",
        run(&client, url, None, None, None, None).await.unwrap()
    );
}

#[test]
fn find_search_result_test() {
    let results: [SearchResult; 2] = [
        SearchResult {
            link: "//kodikplayer.com/video/54982/9c161034342aff5e14dacd613f21c209/720p".to_owned(),
            translation: Translation {
                title: "Reanimedia".to_owned(),
                r#type: TranslationType::Voice,
            },
            seasons: None,
        },
        SearchResult {
            link: "//kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p".to_owned(),
            translation: Translation {
                title: "Subtitles".to_owned(),
                r#type: TranslationType::Subtitles,
            },
            seasons: None,
        },
    ];

    let test_cases = [
        (
            Some("Reanimedia"),
            Some(TranslationType::Subtitles),
            "Reanimedia",
        ),
        (None, None, "Reanimedia"),
        (None, Some(TranslationType::Subtitles), "Subtitles"),
    ];

    for (title, r#type, expected_title) in test_cases {
        assert_eq!(
            expected_title.to_owned(),
            find_search_result(results.to_vec(), title, r#type.as_ref())
                .unwrap()
                .translation
                .title
        );
    }
}
