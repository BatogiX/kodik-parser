use log::{Level, LevelFilter, Log, Metadata, Record};

pub const YELLOW_BOLD: &str = "\x1b[1;33m";
pub const RED_BOLD: &str = "\x1b[1;31m";
pub const RESET: &str = "\x1b[0m";
const BLUE_BOLD: &str = "\x1b[1;34m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

pub fn setup_logging(level_filter: LevelFilter) {
    log::set_logger(&LOGGER).ok();
    log::set_max_level(level_filter);
}

static LOGGER: KodikLogger = KodikLogger;

struct KodikLogger;

impl Log for KodikLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if !record.target().starts_with("kodik") {
            return;
        }

        match record.level() {
            Level::Error => eprintln!("{RED_BOLD}error:{RESET} {BOLD}{}{RESET}", record.args()),
            Level::Warn => eprintln!("{YELLOW_BOLD}warning:{RESET} {BOLD}{}{RESET}", record.args()),
            Level::Info => eprintln!("{BLUE_BOLD}::{RESET} {BOLD}{}{RESET}", record.args()),
            Level::Debug => eprintln!("  {BLUE_BOLD}->{RESET} {}", record.args()),
            Level::Trace => eprintln!("{DIM}{}{RESET}", record.args()),
        }
    }

    fn flush(&self) {}
}
