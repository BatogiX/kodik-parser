use crate::re::extract_domain;

#[test]
fn getting_domain() {
    let url_with_scheme =
        "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let url_without_scheme = "kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";

    assert_eq!("kodikplayer.com", extract_domain(url_with_scheme).unwrap());
    assert_eq!(
        "kodikplayer.com",
        extract_domain(url_without_scheme).unwrap()
    );
}
