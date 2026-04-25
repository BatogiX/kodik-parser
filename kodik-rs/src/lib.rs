use crate::cache::Cache;
use crate::config::{Config, ExecutionMode, Quality};
use kodik_parser::{Response, reqwest::Client};
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
    let mut config = Config::build(args).unwrap_or_else(|e| e.exit());
    logging::setup_logging(config.level_filter());
    let mut cache = Cache::load();

    if let Some(ref cache) = cache {
        cache.apply(&mut config);
    }

    let client = Client::new();

    let exit_code = match config.execution_mode() {
        ExecutionMode::Parallel => run_parallel(&client, config.urls, &config.quality).await,
        ExecutionMode::Lazy => {
            run_lazy(
                &client,
                config.urls,
                &config.quality,
                config.player.as_deref(),
            )
            .await
        }
    };

    if let Some(ref mut cache) = cache
        && cache.is_changed(config.cookie.as_deref())
    {
        log::warn!("Updating cache... in {}", cache.path.display());
        cache.update(config.cookie.as_deref());
        cache.save();
    }

    exit_code
}

async fn run_parallel(client: &Client, urls: Vec<String>, quality: &Quality) -> ExitCode {
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
    quality: &Quality,
    player: Option<&str>,
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

fn get_link<'a>(response: &'a Response, quality: &'a Quality) -> Option<&'a str> {
    let links = match quality {
        Quality::P360 => &response.links.quality_360,
        Quality::P480 => &response.links.quality_480,
        Quality::P720 => &response.links.quality_720,
    };

    links.first().map(|link| link.src.as_str())
}

fn spawn_player(player: &str, link: &str) -> Result<(), String> {
    let mut parts = player.split_whitespace();
    let program = parts.next().ok_or("empty player")?;

    let program = if cfg!(target_os = "windows") && program == "mpv" {
        "mpv.com"
    } else {
        program
    };

    let mut cmd = Command::new(program);
    cmd.args(parts)
        .arg(link)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn().map_err(|e| format!("failed to spawn player '{program}': {e}"))?;
    child.wait().map(|_| ()).map_err(|e| format!("player '{program}' failed: {e}"))
}
