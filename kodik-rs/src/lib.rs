use crate::cache::Cache;
use crate::config::{Config, ExecutionMode, Quality};
use kodik_parser::Response;
use kodik_shiki::TranslationType;
use reqwest::Client;
use reqwest::cookie::CookieStore;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::io::{self, BufWriter};
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;

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
    let config = Config::build(args).unwrap_or_else(|e| e.exit());
    logging::setup_logging(config.level_filter());
    let mut cache = Cache::load();

    if let Some(ref cache) = cache {
        cache.apply();
    }

    let jar = config.load_cookies()?;
    let client = Client::builder().cookie_provider(Arc::new(jar)).build()?;

    match config.execution_mode() {
        ExecutionMode::Parallel => todo!("paralel does not implemented yet"),
        ExecutionMode::Lazy => {
            run_lazy(&client, &config).await?;
        }
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

async fn run_parallel(client: &Client, urls: Vec<String>, quality: &Quality) -> Result<(), Box<dyn Error>> {
    let results = {
        let mut set = tokio::task::JoinSet::new();
        for (idx, url) in urls.into_iter().enumerate() {
            let client = client.clone();
            set.spawn(async move {
                let result = kodik_parser::parse(&client, &url).await;
                (idx, result)
            });
        }

        let mut results = set.join_all().await;
        results.sort_unstable_by_key(|a| a.0);
        results
    };

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    for (_, res) in results {
        let kodik_response = res?;

        let link = get_link(&kodik_response, quality).ok_or("no playable links found for this video")?;

        writeln!(handle, "{link}")?;
    }

    Ok(())
}

async fn run_lazy(client: &Client, config: &Config) -> Result<(), Box<dyn Error>> {
    fn inner(
        kodik_response: &Response,
        config: &Config,
        title: Option<&str>,
        episode: Option<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let link = get_link(kodik_response, &config.quality).ok_or("no playable links found for this video")?;

        if let Some(player) = &config.player {
            spawn_player(player, link, title, episode)?;
        } else {
            writeln!(io::stdout(), "{link}")?;
        }

        Ok(())
    }

    for url in &config.urls {
        match &url.host_str() {
            Some("shikimori.io" | "shikimori.net") => {
                let search_response = kodik_shiki::resolve_anime(client, url.as_str()).await?;

                let search_result = search_response.find_search_result(
                    config.translation_title.as_deref(),
                    config.translation_type.map(TranslationType::from).as_ref(),
                )?;

                let skip = if let Some(skip) = config.episode {
                    skip
                } else {
                    kodik_shiki::fetch_user_rate(client, url.as_str()).await?
                };

                match &search_result.seasons {
                    Some(seasons) => {
                        let (_, season) = seasons.iter().next_back().ok_or("season not found")?;
                        for (episode_number, episode) in season.episodes.iter().skip(skip) {
                            inner(
                                &kodik_parser::parse(client, episode).await?,
                                config,
                                Some(&search_result.title),
                                Some(*episode_number),
                            )?;
                        }
                    }
                    None => inner(
                        &kodik_parser::parse(client, &search_result.link).await?,
                        config,
                        Some(&search_result.title),
                        None,
                    )?,
                }
            }
            Some("kodikplayer.com") => {
                inner(&kodik_parser::parse(client, url.as_str()).await?, config, None, None)?;
            }
            _ => return Err(format!("url '{url}' is not supported").into()),
        }
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
