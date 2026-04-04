use crate::cache::Cache;
use crate::config::{Config, OPTIONS};
use kodik_parser::{Client, KodikError, KodikResponse};
use log::LevelFilter;
use std::io::Write;
use std::io::{self, BufWriter};
use std::process::ExitCode;

mod cache;
mod config;
mod logging;

#[must_use]
pub async fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        eprint!("{}", OPTIONS.help());
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
        eprint!("{}", OPTIONS.help());
        return ExitCode::FAILURE;
    }

    let mut cache = Cache::load();
    if let Some(cache) = cache.as_ref() {
        cache.apply();
    }
    let client = Client::new();

    let results = parallel(config.urls, client).await;

    if let Some(cache) = &mut cache
        && cache.is_changed()
    {
        log::debug!("Updating cache...");
        cache.update();
        cache.save();
    }

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());
    for (_, res) in results {
        let kodik_response = match res {
            Ok(kodik_response) => kodik_response,
            Err(e) => {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        };

        let links = &kodik_response.links;
        if let Some(link) = [&links.quality_720, &links.quality_480, &links.quality_360]
            .iter()
            .find_map(|q| q.first())
        {
            if let Err(e) = writeln!(handle, "{}", link.src) {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        } else {
            log::error!("no playable links found for this video");
            return ExitCode::FAILURE;
        }
    }

    if let Err(e) = handle.flush() {
        log::error!("{e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn parallel(
    urls: Vec<String>,
    client: Client,
) -> Vec<(usize, Result<KodikResponse, KodikError>)> {
    let mut set = tokio::task::JoinSet::new();
    for (idx, url) in urls.into_iter().enumerate() {
        let client = client.clone();
        set.spawn(async move {
            let result = kodik_parser::parser::parse(&client, &url).await;
            (idx, result)
        });
    }

    let mut results = set.join_all().await;
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}
