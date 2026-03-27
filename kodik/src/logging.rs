use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn setup_logging() {
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
