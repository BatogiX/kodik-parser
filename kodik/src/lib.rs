use crate::cache::Cache;
use kodik_parser::Client;
use std::io::Write;
use std::io::{self, BufWriter};
use std::process::ExitCode;

mod cache;
mod logging;

#[must_use]
pub async fn run(args: Vec<String>) -> ExitCode {
    logging::setup_logging();

    if args.len() < 2 {
        log::error!("Usage: {} [URLS]", args[0]);
        return ExitCode::FAILURE;
    }

    let mut cache = Cache::load();
    if let Some(cache) = cache.as_ref() {
        cache.apply().await;
    }
    let client = Client::new();

    let mut set = tokio::task::JoinSet::new();
    for (idx, url) in args.into_iter().skip(1).enumerate() {
        let client = client.clone();
        set.spawn(async move {
            let result = kodik_parser::parser::parse(&client, &url).await;
            (idx, result)
        });
    }

    let mut results = set.join_all().await;
    results.sort_by(|a, b| a.0.cmp(&b.0));

    if let Some(cache) = &mut cache
        && cache.is_changed().await
    {
        log::debug!("Updating cache...");
        cache.update().await;
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
