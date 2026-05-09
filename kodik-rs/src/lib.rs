use crate::cache::Cache;
use crate::config::{Config, ExecutionMode, Quality};
use anyhow::{self, Context as _, Result, bail};
use kodik_parser::Links;
use kodik_shiki::TranslationType;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Url};
use std::fmt::Write as _;
use std::io::{self, BufWriter, Write as _};
use std::process::{Command, ExitCode, Stdio};
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

    match config.execution_mode() {
        ExecutionMode::Parallel => run_parallel(&client, Arc::clone(&config), jar).await?,
        ExecutionMode::Lazy => run_lazy(&client, &config, jar).await?,
    }

    if let Some(ref mut cache) = cache
        && cache.is_changed()
    {
        log::warn!("updating cache... in {}", cache.path.display());
        cache.update();
        cache.save();
    }

    Ok(())
}

async fn run_parallel(client: &Client, config: Arc<Config>, jar: Arc<Jar>) -> crate::Result<()> {
    type Handles = Vec<JoinHandle<crate::Result<Vec<String>>>>;

    async fn future(client: Client, config: Arc<Config>, jar: Arc<Jar>, url: Url) -> crate::Result<Vec<String>> {
        let mut urls_to_parse = Vec::new();

        let mut collect = |url_str: &str, _title: Option<String>, _episode: Option<usize>| {
            urls_to_parse.push(url_str.to_string());
            Ok(())
        };

        resolve_url(&client, &url, &config, &jar, &mut collect).await?;

        let mut parse_handles: Vec<JoinHandle<Result<String>>> = Vec::new();
        for url_str in urls_to_parse {
            let client = client.clone();
            let quality = config.quality;
            parse_handles.push(tokio::spawn(
                async move { parse_link(&client, &url_str, &quality).await },
            ));
        }

        let mut links = Vec::new();
        for handle in parse_handles {
            if let Ok(Ok(link)) = handle.await {
                links.push(link);
            }
        }

        Ok(links)
    }

    let handles: Handles = config
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

async fn run_lazy(client: &Client, config: &Config, jar: Arc<Jar>) -> Result<()> {
    for url in &config.urls {
        let mut resolved = Vec::new();

        {
            let mut collect = |url_str: &str, title: Option<String>, episode: Option<usize>| -> Result<()> {
                resolved.push((url_str.to_owned(), title, episode));
                Ok(())
            };

            resolve_url(client, url, config, &jar, &mut collect).await?;
        }

        for (url_str, title, episode) in resolved {
            let kodik_response = kodik_parser::parse(client, &url_str).await?;
            let link =
                get_link(&kodik_response.links, &config.quality).context("no playable links found for this video")?;

            if let Some(player) = &config.player {
                spawn_player(player, link, title.as_deref(), episode)?;
            } else {
                println!("{link}");
            }
        }
    }

    Ok(())
}

async fn parse_link(client: &Client, url: &str, quality: &Quality) -> Result<String> {
    let kodik_response = kodik_parser::parse(client, url).await?;
    get_link(&kodik_response.links, quality)
        .context("no playable links found for this video")
        .map(std::borrow::ToOwned::to_owned)
}

fn get_link<'a>(links: &'a Links, quality: &'a Quality) -> Option<&'a str> {
    let links = match quality {
        Quality::P360 => &links.quality_360,
        Quality::P480 => &links.quality_480,
        Quality::P720 => &links.quality_720,
    };

    links.first().map(|link| link.src.as_str())
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

fn spawn_player(player: &str, link: &str, title: Option<&str>, episode: Option<usize>) -> Result<()> {
    let mut parts = player.split_whitespace();
    let program = parts.next().context("empty player")?;

    #[cfg(windows)]
    let program = { if program == "mpv" { "mpv.com" } else { program } };

    let mut video_title = String::new();
    if let Some(title) = title {
        let _ = write!(video_title, "{title}");

        if let Some(episode) = episode {
            let _ = write!(video_title, " — Episode {episode}");
        }
    }

    let mut cmd = Command::new(program);

    if !video_title.is_empty() && (program == "mpv" || program == "mpv.com") {
        cmd.arg(format!("--title={video_title}"));
    }

    cmd.args(parts)
        .arg(link)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to spawn player '{program}'"))?
        .wait()
        .map(|_| ())
        .with_context(|| format!("player '{program}' failed"))
}

async fn resolve_shiki<F>(client: &Client, url: &Url, config: &Config, jar: &Jar, fun: &mut F) -> Result<()>
where
    F: FnMut(&str, Option<String>, Option<usize>) -> Result<()>,
{
    let has_cookies = jar.cookies(url).is_some();

    if config.cookies.is_some() && !has_cookies {
        log::warn!("cookies not found for: {url}");
    }

    let shiki_api_animes = if has_cookies || config.player.is_some() {
        Some(kodik_shiki::fetch_shiki_api_animes(client, url.as_str()).await?)
    } else {
        None
    };

    if let Some(mode) = &config.related_mode {
        let related = kodik_shiki::fetch_franchise(client, url.as_str()).await?;
    } else {
        let search_response = kodik_shiki::resolve_anime(client, url.as_str()).await?;

        let search_result = search_response.find_search_result(
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
            for (episode_number, episode) in season.episodes.iter().skip(skip) {
                if config.player.is_some() {
                    if let Some(ref shiki_api_animes) = shiki_api_animes {
                        fun(episode, Some(shiki_api_animes.name.clone()), Some(*episode_number))?;
                    } else {
                        fun(episode, Some(search_result.title.clone()), Some(*episode_number))?;
                    }
                } else {
                    fun(episode, None, None)?;
                }
            }
        } else {
            if skip > 0 {
                return Ok(());
            }

            let episode = &search_result.link;
            if config.player.is_some() {
                if let Some(ref shiki_api_animes) = shiki_api_animes {
                    fun(episode, Some(shiki_api_animes.name.clone()), None)?;
                } else {
                    fun(episode, Some(search_result.title.clone()), None)?;
                }
            } else {
                fun(episode, None, None)?;
            }
        }
    }

    Ok(())
}
