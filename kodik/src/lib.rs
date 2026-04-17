use crate::cache::Cache;
use crate::config::{COMMAND, Config, Quality};
use kodik_parser::{Response, reqwest::Client};
use log::LevelFilter;
use std::io::Write;
use std::io::{self, BufWriter};
use std::process::{Command, ExitCode, Stdio};

mod cache;
mod config;
mod logging;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

#[must_use]
pub async fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        eprint!("{}", COMMAND.help());
        return ExitCode::FAILURE;
    }

    let mut config = match Config::build(args) {
        Ok(config) => {
            logging::setup_logging(config.level_filter);
            config
        }
        Err(err) => {
            logging::setup_logging(LevelFilter::Info);
            log::error!("{err}");
            return ExitCode::FAILURE;
        }
    };

    if config.help {
        eprint!("{}", COMMAND.help());
        return ExitCode::FAILURE;
    }

    let mut cache_opt = Cache::load();
    if let Some(cache) = cache_opt.as_ref() {
        cache.apply(&mut config);
    }

    let client = Client::new();
    let use_lazy = config.lazy || config.player.is_some();

    let mut idx = 0;
    while idx < config.urls.len() {
        let Some(url) = config.urls.get(idx) else {
            break;
        };

        if url.starts_with("https://shiki") {
            match kodik_shiki::run(
                &client,
                url,
                config.cookie.as_deref(),
                config.translation_title.as_deref(),
                config.translation_type.0.as_ref(),
                config.episode,
            )
            .await
            {
                Ok(video_result) => match video_result {
                    kodik_shiki::VideoResult::Episodes(episodes) => {
                        let episode_count = episodes.len();
                        config.urls.splice(idx..=idx, episodes);
                        idx += episode_count;
                    }
                    kodik_shiki::VideoResult::Film(film) => {
                        if let Some(url_ref) = config.urls.get_mut(idx) {
                            *url_ref = film;
                        }
                        idx += 1;
                    }
                },
                Err(e) => {
                    log::error!("{e}");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            idx += 1;
        }
    }

    let exit_code = if use_lazy {
        run_lazy(&client, config.urls, config.quality, config.player).await
    } else {
        run_parallel(&client, config.urls, config.quality).await
    };

    if let Some(cache) = cache_opt.as_mut()
        && cache.is_changed(config.cookie.as_deref())
    {
        log::warn!("Updating cache... in {}", cache.path.display());
        cache.update(config.cookie.as_deref());
        cache.save();
    }

    exit_code
}

async fn run_parallel(client: &Client, urls: Vec<String>, quality: Quality) -> ExitCode {
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
        results.sort_by_key(|a| a.0);
        results
    };

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    for (_, res) in results {
        let kodik_response = match res {
            Ok(r) => r,
            Err(e) => {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        };

        let Some(link) = get_link(&kodik_response, quality) else {
            log::error!("no playable links found for this video");
            return ExitCode::FAILURE;
        };

        if let Err(e) = writeln!(handle, "{link}") {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    }

    if let Err(e) = handle.flush() {
        log::error!("{e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run_lazy(
    client: &Client,
    urls: Vec<String>,
    quality: Quality,
    player: Option<String>,
) -> ExitCode {
    for url in urls {
        let kodik_response = match kodik_parser::parse(client, &url).await {
            Ok(r) => r,
            Err(e) => {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        };

        let Some(link) = get_link(&kodik_response, quality) else {
            log::error!("no playable links found for this video");
            return ExitCode::FAILURE;
        };

        if let Some(player) = &player {
            if let Err(e) = spawn_player(player, link) {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        } else if let Err(e) = writeln!(io::stdout(), "{link}") {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn get_link(response: &Response, quality: Quality) -> Option<&str> {
    match quality {
        Quality::P360 => response
            .links
            .quality_360
            .first()
            .map(|link| link.src.as_str()),
        Quality::P480 => response
            .links
            .quality_480
            .first()
            .map(|link| link.src.as_str()),
        Quality::P720 => response
            .links
            .quality_720
            .first()
            .map(|link| link.src.as_str()),
    }
}

fn spawn_player(player: &str, link: &str) -> Result<(), String> {
    let mut parts = player.split_whitespace();
    let mut program = parts.next().ok_or("empty player")?;

    // Patch for Windows to terminate mpv with Ctrl+C
    program = if cfg!(target_os = "windows") && program == "mpv" {
        "mpv.com"
    } else {
        program
    };

    let mut cmd = Command::new(program);
    for arg in parts {
        cmd.arg(arg);
    }

    cmd.arg(link)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .and_then(|mut child| child.wait().map(|_| ()))
        .map_err(|e| format!("failed to spawn player '{program}': {e}"))
}
