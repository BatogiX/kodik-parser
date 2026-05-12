use crate::cache::Cache;
use crate::config::{Config, Quality};
use anyhow::{self, Context as _, Result, bail};
use kodik_shiki::{ShikiApiAnimes, TranslationType};
use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Url};
use std::io::{self, BufWriter, Write as _};
use std::process::ExitCode;
use std::sync::Arc;
use tokio::task::JoinHandle;

mod cache;
mod config;
mod logging;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

pub async fn run(args: Vec<String>) -> ExitCode {
    match run_impl(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

async fn run_impl(args: Vec<String>) -> Result<()> {
    let config = Arc::new(Config::build(args).unwrap_or_else(|e| e.exit()));
    logging::setup_logging(config.level_filter());
    let mut cache = Cache::load();

    if let Some(ref cache) = cache {
        cache.apply();
    }

    let jar = Arc::new(config.load_cookies()?);
    let client = Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .gzip(true)
        .brotli(true)
        .zstd(true)
        .deflate(true)
        .build()?;

    run_parallel(&client, Arc::clone(&config), jar).await?;

    if let Some(ref mut cache) = cache
        && cache.is_changed()
    {
        log::warn!("updating cache... in {}", cache.path.display());
        cache.update();
        cache.save();
    }

    Ok(())
}

async fn run_parallel(client: &Client, config: Arc<Config>, jar: Arc<Jar>) -> Result<()> {
    async fn future(client: Client, config: Arc<Config>, jar: Arc<Jar>, url: Url) -> Result<Vec<String>> {
        let mut urls = Vec::new();

        let mut collect = |url: &str, _title: Option<String>, _episode: Option<usize>| {
            urls.push(url.to_string());
            Ok(())
        };

        resolve_url(&client, &url, &config, &jar, &mut collect).await?;

        let mut parse_handles: Vec<JoinHandle<Result<String>>> = Vec::new();
        for url in urls {
            let client = client.clone();
            let quality = config.quality;
            parse_handles.push(tokio::spawn(async move {
                let kodik_response = kodik_parser::parse(&client, url.as_str()).await?;
                let links = match quality {
                    Quality::P360 => &kodik_response.links.quality_360,
                    Quality::P480 => &kodik_response.links.quality_480,
                    Quality::P720 => &kodik_response.links.quality_720,
                };

                links
                    .first()
                    .map(|link| link.src.as_str())
                    .context("no playable links found for this video")
                    .map(std::borrow::ToOwned::to_owned)
            }));
        }

        let mut links = Vec::new();
        for handle in parse_handles {
            if let Ok(Ok(link)) = handle.await {
                links.push(link);
            }
        }

        Ok(links)
    }

    let handles: Vec<JoinHandle<Result<Vec<String>>>> = config
        .urls
        .iter()
        .cloned()
        .map(|url| tokio::spawn(future(client.clone(), Arc::clone(&config), Arc::clone(&jar), url)))
        .collect();

    let mut all_links = Vec::new();
    for handle in handles {
        let links = handle.await??;
        all_links.extend(links);
    }

    let mut stdout = BufWriter::new(io::stdout());
    for link in all_links {
        writeln!(stdout, "{link}")?;
    }
    stdout.flush()?;

    Ok(())
}

async fn resolve_url<F>(client: &Client, url: &Url, config: &Config, jar: &Jar, fun: &mut F) -> Result<()>
where
    F: FnMut(&str, Option<String>, Option<usize>) -> Result<()>,
{
    match url
        .host_str()
        .with_context(|| format!("url '{url}' is not supported"))?
        .split_once('.')
        .with_context(|| format!("url '{url}' is not supported"))?
        .0
    {
        "shikimori" => resolve_shiki(client, url, config, jar, fun).await?,
        "kodikplayer" => fun(url.as_str(), None, None)?,
        _ => bail!("url '{url}' is not supported"),
    }

    Ok(())
}

async fn resolve_shiki<F>(client: &Client, url: &Url, config: &Config, jar: &Jar, fun: &mut F) -> Result<()>
where
    F: FnMut(&str, Option<String>, Option<usize>) -> Result<()>,
{
    let shikimori_id = kodik_shiki::extract_id(url.as_str())?;
    let has_cookies = jar.cookies(url).is_some();

    if config.cookies.is_some() && !has_cookies {
        log::warn!("cookies not found for: {url}");
    }

    let shiki_api_animes = if has_cookies || config.related_mode.is_some() {
        Some(kodik_shiki::fetch_shiki_api_animes(client, url.as_str()).await?)
    } else {
        None
    };

    if let Some(ref _mode) = config.related_mode {
        if let Some(shiki_api_animes) = shiki_api_animes.as_ref() {
            let Some(franchise) = shiki_api_animes.franchise.as_deref() else {
                return shiki_helper(client, url, config, shikimori_id, Some(shiki_api_animes), fun).await;
            };

            let domain = url.domain().context("url have no domain")?;
            let not_anime_ids = kodik_shiki::fetch_not_anime_ids(client, franchise)
                .await?
                .context("there are no 'not anime ids (just log::warn)'")?;

            let mut related = kodik_shiki::Related::fetch_by_franchise(client, franchise, domain).await?;
            let animes = &related.filter_by_not_anime_ids(not_anime_ids)?.sort_by_chrono().animes;

            for anime in animes {
                println!("{}", anime.name);
            }

            for anime in animes {
                shiki_helper(client, url, config, &anime.id, Some(shiki_api_animes), fun).await?;
            }
        }
    } else {
        shiki_helper(client, url, config, shikimori_id, shiki_api_animes.as_ref(), fun).await?;
    }

    Ok(())
}

async fn shiki_helper<F>(
    client: &Client,
    url: &Url,
    config: &Config,
    shikimori_id: &str,
    shiki_api_animes: Option<&ShikiApiAnimes>,
    fun: &mut F,
) -> Result<()>
where
    F: FnMut(&str, Option<String>, Option<usize>) -> Result<()>,
{
    let kodik_api_resp = kodik_shiki::fetch_kodik_videos(client, shikimori_id).await?;

    let search_result = kodik_api_resp.find_search_result(
        config.translation_title.as_deref(),
        config.translation_type.map(TranslationType::from).as_ref(),
    )?;

    let skip = shiki_api_animes.as_ref().map_or(0, |shiki_api_animes| {
        shiki_api_animes.user_rate.as_ref().map_or_else(
            || {
                log::warn!("user rate not found for: {url}, defaulting to first episode");
                0
            },
            |user_rate| user_rate.episodes,
        )
    });

    if let Some(seasons) = &search_result.seasons {
        let (_, season) = seasons.iter().next_back().context("season not found")?;
        for (_episode_number, episode) in season.episodes.iter().skip(skip) {
            fun(episode, None, None)?;
        }
    } else {
        if skip > 0 {
            return Ok(());
        }

        let episode = &search_result.link;
        fun(episode, None, None)?;
    }

    Ok(())
}
