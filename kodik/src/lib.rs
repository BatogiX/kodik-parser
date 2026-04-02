use crate::cache::Cache;
use kodik_parser::Client;
use std::fmt::Write;
use std::io::IsTerminal;
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

    for url in args.into_iter().skip(1) {
        let client = client.clone();
        set.spawn(async move { kodik_parser::parser::parse(&client, &url).await });
    }

    let results = set.join_all().await;

    if let Some(cache) = &mut cache
        && cache.is_changed().await
    {
        log::debug!("Updating cache...");
        cache.update().await;
        cache.save();
    }

    let mut stdout = String::new();
    for res in results {
        let kodik_response = match res {
            Ok(kodik_response) => kodik_response,
            Err(e) => {
                log::error!("{e}");
                return ExitCode::FAILURE;
            }
        };

        let links = &kodik_response.links;
        if let Some(link) = links
            .quality_720
            .first()
            .or_else(|| links.quality_480.first())
            .or_else(|| links.quality_360.first())
        {
            let _ = write!(stdout, "{} ", link.src);
        } else {
            log::error!("no playable links found for this video");
            return ExitCode::FAILURE;
        }
    }

    if std::io::stdout().is_terminal() {
        log::info!("{stdout}");
    } else {
        print!("{stdout} ",);
    }

    ExitCode::SUCCESS
}
