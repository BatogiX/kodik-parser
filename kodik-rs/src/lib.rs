use crate::cache::Cache;
use crate::config::{Config, ExecutionMode, Quality};
use kodik_parser::Response;
use kodik_shiki::TranslationType;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Url};
use std::error::Error;
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

pub async fn run_impl(args: Vec<String>) -> Result<(), Box<dyn Error>> {
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
        ExecutionMode::Parallel => run_parallel(&client, Arc::clone(&config), Arc::clone(&jar)).await?,
        ExecutionMode::Lazy => run_lazy(&client, &config, Arc::clone(&jar)).await?,
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

async fn run_parallel(client: &Client, config: Arc<Config>, jar: Arc<Jar>) -> Result<(), Box<dyn Error>> {
    let mut handles: Vec<JoinHandle<Result<Vec<String>, Box<dyn Error + Send + Sync + 'static>>>> = Vec::new();

    for url in config.urls.iter().cloned() {
        let client = client.clone();
        let config = Arc::clone(&config);
        let jar = Arc::clone(&jar);

        let handle = tokio::spawn(async move {
            let mut links = Vec::new();

            let mut collect = |kodik_response, _title, _episode| -> Result<(), Box<dyn Error>> {
                let link =
                    get_link(&kodik_response, &config.quality).ok_or("no playable links found for this video")?;

                links.push(link.to_owned());
                Ok(())
            };

            resolve_url(&client, &url, config.as_ref(), &jar, &mut collect).await;

            Ok(links)
        });

        handles.push(handle);
    }

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for handle in handles {
        let links = handle.await?.unwrap();

        for link in links {
            writeln!(out, "{link}")?;
        }
    }

    // for links in results {
    //     for link in links {
    //         writeln!(handle, "{link}")?;
    //     }
    // }

    Ok(())
}

async fn run_lazy(client: &Client, config: &Config, jar: Arc<Jar>) -> Result<(), Box<dyn Error>> {
    fn spawn_player(player: &str, link: &str, title: Option<&str>, episode: Option<usize>) -> Result<(), String> {
        let mut parts = player.split_whitespace();
        let program = parts.next().ok_or("empty player")?;

        let program = if cfg!(target_os = "windows") && program == "mpv" {
            "mpv.com"
        } else {
            program
        };

        let mut video_title = String::new();
        if let Some(title) = title {
            video_title.push_str(title);

            if let Some(episode) = episode {
                let _ = write!(video_title, " — Episode {episode}");
            }
        }

        let mut cmd = Command::new(program);
        cmd.args(parts);

        if !video_title.is_empty() && (program == "mpv" || program == "mpv.com") {
            cmd.arg(format!("--title={video_title}"));
        }

        cmd.arg(link)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn player '{program}': {e}"))?;
        child
            .wait()
            .map(|_| ())
            .map_err(|e| format!("player '{program}' failed: {e}"))
    }

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    let mut clos = |kodik_response, title: Option<String>, episode| -> Result<(), Box<dyn Error>> {
        let link = get_link(&kodik_response, &config.quality).ok_or("no playable links found for this video")?;

        if let Some(player) = &config.player {
            spawn_player(player, link, title.as_deref(), episode)?;
        } else {
            writeln!(handle, "{link}")?;
        }

        Ok(())
    };

    for url in &config.urls {
        resolve_url(client, url, config, &jar, &mut clos).await?
    }

    Ok(())
}

fn get_link<'a>(response: &'a Response, quality: &'a Quality) -> Option<&'a str> {
    let links = match quality {
        Quality::P360 => &response.links.quality_360,
        Quality::P480 => &response.links.quality_480,
        Quality::P720 => &response.links.quality_720,
    };

    links.first().map(|link| link.src.as_str())
}

async fn resolve_url<F>(
    client: &Client,
    url: &Url,
    config: &Config,
    jar: &Arc<Jar>,
    inner: &mut F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(kodik_parser::Response, Option<String>, Option<usize>) -> Result<(), Box<dyn Error>>,
{
    match url
        .host_str()
        .ok_or_else(|| format!("url '{url}' is not supported"))?
        .split_once('.')
        .ok_or_else(|| format!("url '{url}' is not supported"))?
        .0
    {
        "shikimori" => {
            let search_response = kodik_shiki::resolve_anime(client, url.as_str()).await?;

            let search_result = search_response.find_search_result(
                config.translation_title.as_deref(),
                config.translation_type.map(TranslationType::from).as_ref(),
            )?;

            let skip = if jar.cookies(url).is_some() {
                kodik_shiki::fetch_user_rate(client, url.as_str()).await?.unwrap_or(0)
            } else {
                0
            };

            match &search_result.seasons {
                Some(seasons) => {
                    let (_, season) = seasons.iter().next_back().ok_or("season not found")?;
                    for (episode_number, episode) in season.episodes.iter().skip(skip) {
                        inner(
                            kodik_parser::parse(client, episode).await?,
                            Some(search_result.title.clone()),
                            Some(*episode_number),
                        )?;
                    }
                }
                None => inner(
                    kodik_parser::parse(client, &search_result.link).await?,
                    Some(search_result.title.clone()),
                    None,
                )?,
            }
        }
        "kodikplayer" => inner(kodik_parser::parse(client, url.as_str()).await?, None, None)?,
        _ => return Err(format!("url '{url}' is not supported").into()),
    }

    Ok(())
}
