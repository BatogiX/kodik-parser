use log::LevelFilter;
use std::fmt::Write;

use crate::logging::{
    CYAN, CYAN_HIGH_INTENSITY_BOLD, GREEN_HIGH_INTENSITY_BOLD, RESET, YELLOW_BOLD,
};

pub static OPTIONS: Options = Options::default();

struct OptionHelp {
    flags: &'static str,
    description: &'static str,
}

pub struct Options([OptionHelp; 5]);

impl Options {
    const fn default() -> Self {
        Self([
            OptionHelp {
                flags: "-l, --lazy",
                description: "Outputs one by one (turns off parallelism)",
            },
            OptionHelp {
                flags: "-p, --player <MEDIA-PLAYER>",
                description: "Specify media player (implies --lazy)",
            },
            OptionHelp {
                flags: "-v, --verbose",
                description: "Use verbose output (-vv very verbose)",
            },
            OptionHelp {
                flags: "-q, --quiet",
                description: "Do not print log messages",
            },
            OptionHelp {
                flags: "-h, --help",
                description: "Print help",
            },
        ])
    }

    pub fn help(&self) -> String {
        let width = self.0.iter().map(|opt| opt.flags.len()).max().unwrap_or(0);
        let mut help = format!(
            "Kodik parser to get direct links on videos\n
{GREEN_HIGH_INTENSITY_BOLD}Usage:{RESET} {CYAN_HIGH_INTENSITY_BOLD}kodik{RESET} {CYAN}[URLS]{RESET}\n
{GREEN_HIGH_INTENSITY_BOLD}Options:{RESET}",
        );

        for opt in &self.0 {
            let _ = write!(
                help,
                "\n  {CYAN_HIGH_INTENSITY_BOLD}{:width$}{RESET}  {}",
                opt.flags, opt.description
            );
        }

        help
    }
}

pub struct Config {
    pub urls: Vec<String>,
    pub level_filter: LevelFilter,
    pub lazy: bool,
    pub help: bool,
    pub player: Option<String>,
}

impl Config {
    pub fn build(args: Vec<String>) -> Result<Self, String> {
        let mut urls = Vec::new();
        let mut level_filter = LevelFilter::Info;
        let mut lazy = false;
        let mut help = false;
        let mut player = None;

        let mut args = args.into_iter().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-v" | "--verbose" => level_filter = LevelFilter::Debug,
                "-vv" => level_filter = LevelFilter::Trace,
                "-q" | "--quiet" => level_filter = LevelFilter::Off,
                "-l" | "--lazy" => lazy = true,
                "-p" | "--player" => {
                    if let Some(p) = args.next() {
                        player = Some(p);
                    } else {
                        return Err(format!(
                            "a value is required for '{YELLOW_BOLD}--player <MEDIA-PLAYER>{RESET}' but was not supplied",
                        ));
                    }
                }
                "-h" | "--help" => help = true,
                _ => {
                    if arg.starts_with('-') {
                        return Err(format!(
                            "unexpected argument '{YELLOW_BOLD}{arg}{RESET}' found\n
{GREEN_HIGH_INTENSITY_BOLD}Usage:{RESET} {CYAN_HIGH_INTENSITY_BOLD}kodik{RESET} {CYAN}[URLS]{RESET}\n
For more information, try '{CYAN_HIGH_INTENSITY_BOLD}--help{RESET}'.",
                        ));
                    }
                    urls.push(arg);
                }
            }
        }

        Ok(Self {
            urls,
            level_filter,
            lazy,
            help,
            player,
        })
    }
}
