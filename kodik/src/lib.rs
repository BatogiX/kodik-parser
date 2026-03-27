use env_logger::Builder;
use kodik_parser::Agent;
use log::{Level, LevelFilter};
use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use crate::cache::KodikCache;

mod cache;

fn setup_logging() {
    Builder::new()
        .format(|buf, record| match record.level() {
            Level::Info => writeln!(buf, "\x1b[1;34m::\x1b[0m \x1b[1m{}\x1b[0m", record.args()),
            Level::Debug => writeln!(buf, "  \x1b[1;34m->\x1b[0m {}", record.args()),
            Level::Warn => writeln!(
                buf,
                "\x1b[1;33mwarning:\x1b[0m \x1b[1m{}\x1b[0m",
                record.args()
            ),
            Level::Error => writeln!(
                buf,
                "\x1b[1;31merror:\x1b[0m \x1b[1m{}\x1b[0m",
                record.args()
            ),
            Level::Trace => writeln!(buf, "\x1b[2m{}\x1b[0m", record.args()),
        })
        .filter(None, LevelFilter::Off)
        .filter_module("", LevelFilter::Info)
        .filter_module("kodik_parser", LevelFilter::Trace)
        .init();
}

#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    setup_logging();

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
