use crate::cache::Cache;
use crate::config::{COMMAND, Config, Quality};
use kodik_parser::{Client, KodikResponse};
use log::LevelFilter;
use std::io::Write;
use std::io::{self, BufWriter};
use std::process::{Command, ExitCode, Stdio};

mod cache;
mod config;
mod logging;

#[must_use]
pub async fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        eprint!("{}", COMMAND.help());
        return ExitCode::FAILURE;
    }

    let config = match Config::build(args) {
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

    let mut cache = Cache::load();
    if let Some(cache) = cache.as_ref() {
        cache.apply();
    }

    let client = Client::new();
    let use_lazy = config.lazy || config.player.is_some();

    let exit_code = if use_lazy {
        run_lazy(config, &client).await
    } else {
        run_parallel(config, &client).await
    };

    if let Some(cache) = &mut cache
        && cache.is_changed()
    {
        log::debug!("Updating cache...");
        cache.update();
        cache.save();
    }

    exit_code
}

async fn run_parallel(config: Config, client: &Client) -> ExitCode {
    let results = {
        let mut set = tokio::task::JoinSet::new();
        for (idx, url) in config.urls.into_iter().enumerate() {
            let client = client.clone();
            set.spawn(async move {
                let result = kodik_parser::parse(&client, &url).await;
                (idx, result)
            });
        }

        let mut results = set.join_all().await;
        results.sort_by(|a, b| a.0.cmp(&b.0));
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

        let Some(link) = get_link(&kodik_response, config.quality) else {
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

async fn run_lazy(config: Config, client: &Client) -> ExitCode {
    for url in config.urls {
        let kodik_response = match kodik_parser::parse(client, &url).await {
            Ok(r) => r,
            Err(e) => {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        };

        let Some(link) = get_link(&kodik_response, config.quality) else {
            log::error!("no playable links found for this video");
            return ExitCode::FAILURE;
        };

        if let Some(player) = &config.player {
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

fn get_link(response: &KodikResponse, quality: Quality) -> Option<&str> {
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
    Command::new(player)
        .arg(link)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .and_then(|mut child| child.wait().map(|_| ()))
        .map_err(|e| format!("failed to spawn player '{player}': {e}"))
}
