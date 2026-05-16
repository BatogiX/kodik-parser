use crate::config::Config;
use crate::config::RelatedMode;
use anyhow::Context;
use anyhow::Result;
use kodik_shiki::ShikiApiAnimes;
use kodik_shiki::TranslationType;
use reqwest::cookie::CookieStore;
use reqwest::{Client, Url, cookie::Jar};

pub async fn resolve_shiki(client: &Client, url: &Url, config: &Config, jar: &Jar) -> Result<Vec<String>> {
    let shikimori_id = kodik_shiki::extract_id(url.as_str())?.parse().unwrap();
    let has_cookies = jar.cookies(url).is_some();

    if config.cookies.is_some() && !has_cookies {
        log::warn!("cookies not found for: {url}");
    }

    let shiki_api_animes = if has_cookies || config.related_mode.is_some() {
        Some(kodik_shiki::fetch_shiki_api_animes(client, url.as_str()).await?)
    } else {
        None
    };

    if let Some(ref mode) = config.related_mode {
        if let Some(shiki_api_animes) = shiki_api_animes.as_ref() {
            let Some(franchise) = shiki_api_animes.franchise.as_deref() else {
                return shiki_helper(client, url, config, shikimori_id, Some(shiki_api_animes)).await;
            };

            let domain = url.domain().context("url have no domain")?;

            let mut related = match mode {
                RelatedMode::All => kodik_shiki::Related::fetch_by_franchise(client, franchise, domain, &[]).await?,
                RelatedMode::Essential => {
                    let not_anime_ids = kodik_shiki::fetch_not_anime_ids(client, franchise)
                        .await?
                        .context("there are no 'not anime ids (just log::warn)'")?;

                    kodik_shiki::Related::fetch_by_franchise(client, franchise, domain, not_anime_ids).await?
                }
            };
            related.sort_by_chrono();

            let mut links = Vec::new();
            for anime in &related.animes {
                links.append(&mut shiki_helper(client, url, config, anime.id, Some(shiki_api_animes)).await?);
            }
            return Ok(links);
        }
    } else {
        return shiki_helper(client, url, config, shikimori_id, shiki_api_animes.as_ref()).await;
    }

    Ok(vec![])
}

async fn shiki_helper(
    client: &Client,
    url: &Url,
    config: &Config,
    shikimori_id: usize,
    shiki_api_animes: Option<&ShikiApiAnimes>,
) -> Result<Vec<String>> {
    let kodik_api_resp = kodik_shiki::fetch_kodik_videos(client, shikimori_id).await?;

    let search_result = kodik_api_resp
        .find_search_result(
            config.translation_title.as_deref(),
            config.translation_type.map(TranslationType::from).as_ref(),
        )?
        .to_owned();

    let skip = shiki_api_animes.map_or(0, |shiki_api_animes| {
        shiki_api_animes.user_rate.as_ref().map_or_else(
            || {
                if config.cookies.is_some() {
                    log::warn!("user rate not found for: {url}, defaulting to first episode");
                }
                0
            },
            |user_rate| user_rate.episodes,
        )
    });

    let mut links = Vec::new();
    if let Some(seasons) = search_result.seasons {
        let (_, season) = seasons.into_iter().next_back().context("season not found")?;
        for (_, link) in season.episodes.into_iter().skip(skip) {
            links.push(link);
        }
    } else {
        if skip > 0 {
            return Ok(links);
        }

        let link = search_result.link;
        links.push(link);
    }

    Ok(links)
}
