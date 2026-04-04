use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub const CYAN: &str = "\x1b[0;36m";
pub const GREEN_HIGH_INTENSITY_BOLD: &str = "\x1b[1;92m";
pub const CYAN_HIGH_INTENSITY_BOLD: &str = "\x1b[1;96m";
pub const YELLOW_BOLD: &str = "\x1b[1;33m";
const BLUE_BOLD: &str = "\x1b[1;34m";
pub const RED_BOLD: &str = "\x1b[1;31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

pub fn setup_logging(level_filter: LevelFilter) {
    Builder::new()
        .format(|buf, record| match record.level() {
            Level::Info => writeln!(buf, "{BLUE_BOLD}::{RESET} {BOLD}{}{RESET}", record.args()),
            Level::Debug => writeln!(buf, "  {BLUE_BOLD}->{RESET} {}", record.args()),
            Level::Warn => writeln!(
                buf,
                "{YELLOW_BOLD}warning:{RESET} {BOLD}{}{RESET}",
                record.args()
            ),
            Level::Error => writeln!(
                buf,
                "{RED_BOLD}error:{RESET} {BOLD}{}{RESET}",
                record.args()
            ),
            Level::Trace => writeln!(buf, "{DIM}{}{RESET}", record.args()),
        })
        .filter(None, LevelFilter::Off)
        .filter_module("", level_filter)
        .filter_module("kodik_parser", level_filter)
        .init();
}
