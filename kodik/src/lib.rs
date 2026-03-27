use kodik_parser::Agent;
use std::io::IsTerminal;
use std::process::ExitCode;

use crate::cache::KodikCache;

mod cache;
mod logging;

#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    logging::setup_logging();

    if args.len() < 2 {
        log::error!("Usage: {} <url>", args[0]);
        return ExitCode::FAILURE;
    }
    if args.len() > 2 {
        log::error!(
            "unexpected argument '\x1b[93m{}\x1b[0m' found\n\nUsage: {} <url>",
            args[2],
            args[0]
        );
        return ExitCode::FAILURE;
    }
    let url = &args[1];

    let mut cache = KodikCache::load();
    cache.as_ref().map(KodikCache::apply_to_globals);
    let agent = Agent::new_with_defaults();

    let kodik_response = match kodik_parser::blocking::parse(&agent, url) {
        Ok(kodik_response) => kodik_response,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    if let Some(cache) = &mut cache
        && cache.is_changed()
    {
        log::debug!("Updating cache...");
        cache.update();
    }

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
            println!("{}", link.src);
        }
    } else {
        log::error!("no playable links found for this video");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
