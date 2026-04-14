use std::env;

use reqwest::Client;

use crate::scraper::{get_kodik_videos, get_user_rate, run};

#[tokio::test]
async fn get_user_rate_test() {
    let client = Client::new();
    let domain = "shikimori.io";
    let id = "43";
    let cookie = env::var("_kawai_session").unwrap();

    get_user_rate(&client, domain, id, &cookie)
        .await
        .unwrap()
        .unwrap();
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
    let cookie = env::var("_kawai_session").unwrap();

    println!("{:#?}", run(&client, url, Some(cookie)).await.unwrap());
}

#[tokio::test]
async fn run_film_test() {
    let client = Client::new();
    let url = "https://shikimori.io/animes/43-koukaku-kidoutai";
    let cookie = env::var("_kawai_session").unwrap();

    println!("{:#?}", run(&client, url, Some(cookie)).await.unwrap());
}
