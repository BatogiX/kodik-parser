use crate::parser::extract_id;

#[test]
fn extract_id_shiki_test() {
    let url = "https://shikimori.io/animes/431-howl-no-ugoku-shiro";
    let expected = "431";

    let id = extract_id(url).unwrap();
    assert_eq!(id, expected);
}

#[test]
fn extract_id_shiki_old_test() {
    let urls: [&str; 3] = [
        "https://shikimori.io/animes/z199-sen-to-chihiro-no-kamikakushi",
        "https://shikimori.io/animes/y28851-koe-no-katachi",
        "https://shikimori.io/animes/x16782-kotonoha-no-niwa",
    ];
    let expected: [&str; 3] = ["199", "28851", "16782"];

    for (url, expected) in urls.into_iter().zip(expected) {
        let id = extract_id(url).unwrap();
        assert_eq!(id, expected);
    }
}

#[test]
fn extract_id_mal_test() {
    let url = "https://myanimelist.net/anime/199/Sen_to_Chihiro_no_Kamikakushi";
    let expected = "199";

    let id = extract_id(url).unwrap();
    assert_eq!(id, expected);
}
