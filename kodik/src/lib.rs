use kodik_parser::Agent;
use std::io::IsTerminal;
use std::process::ExitCode;
use std::thread;

use crate::cache::Cache;

mod cache;
mod logging;

#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    logging::setup_logging();

    if args.len() < 2 {
        log::error!("Usage: {} [URLS]", args[0]);
        return ExitCode::FAILURE;
    }

    let mut cache = Cache::load();
    cache.as_ref().map(Cache::apply);
    let agent = Agent::new_with_defaults();

    let results = thread::scope(|s| {
        let mut handles = vec![];

        for url in &args[1..] {
            handles.push(s.spawn(|| kodik_parser::blocking::parse(&agent, url)));
        }

        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect::<Vec<_>>()
    });

    if let Some(cache) = &mut cache
        && cache.is_changed()
    {
        log::debug!("Updating cache...");
        cache.update();
        cache.save();
    }

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
            if std::io::stdout().is_terminal() {
                log::info!("{}", link.src);
            } else {
                print!("{} ", link.src);
            }
        } else {
            log::error!("no playable links found for this video");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
