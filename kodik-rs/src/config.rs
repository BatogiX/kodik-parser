use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use clap::{ArgAction, Parser, ValueEnum, builder::styling};
use kodik_shiki::TranslationType;
use log::LevelFilter;
use reqwest::{Url, cookie::Jar};

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::BrightGreen.on_default().bold())
    .usage(styling::AnsiColor::BrightGreen.on_default().bold())
    .literal(styling::AnsiColor::BrightCyan.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

#[derive(Parser, Debug)]
#[command(name = "kodik", styles = STYLES, arg_required_else_help = true, about = "CLI tool for getting direct links to files from Kodik")]
pub struct Config {
    /// Url(s) to parse
    #[arg(value_name = "URL", required = true)]
    pub urls: Vec<String>,

    /// Outputs one by one (turns off parallelism)
    #[arg(short = 'l', long)]
    pub lazy: bool,

    /// Specify media player (implies --lazy)
    #[arg(short = 'p', long, value_name = "MEDIA-PLAYER")]
    pub player: Option<String>,

    /// Use verbose output (-vv very verbose)
    #[arg(short = 'v', long, action = ArgAction::Count, default_value = "0")]
    pub verbose: u8,

    /// Do not print log messages
    #[arg(short = 's', long, conflicts_with = "verbose")]
    pub silent: bool,

    /// Specify video quality
    #[arg(short = 'q', long, value_name = "QUALITY", default_value = "720")]
    pub quality: Quality,

    /// Specify from which episode start with
    #[arg(short = 'e', long, value_name = "EPISODE")]
    pub episode: Option<usize>,

    /// Specify translation title
    #[arg(long, value_name = "TITLE")]
    pub translation_title: Option<String>,

    /// Specify translation type
    #[arg(long, value_name = "TYPE")]
    pub translation_type: Option<TranslationTypeArg>,

    /// Netscape formatted file to read cookies from
    #[arg(long, value_name = "FILE")]
    pub cookies: Option<String>,

    /// Expand a media database URL into all related URLs
    #[arg(long, value_name = "MODE", default_value = "none")]
    pub related_mode: RelatedMode,
}

impl Config {
    pub fn build(args: Vec<String>) -> Result<Self, clap::Error> {
        Self::try_parse_from(args)
    }

    pub const fn execution_mode(&self) -> ExecutionMode {
        if self.lazy || self.player.is_some() {
            ExecutionMode::Lazy
        } else {
            ExecutionMode::Parallel
        }
    }

    pub const fn level_filter(&self) -> LevelFilter {
        if self.silent {
            LevelFilter::Off
        } else {
            match self.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            }
        }
    }

    pub fn load_cookies(&self) -> Result<Jar, Box<dyn Error>> {
        let jar = Jar::default();

        if let Some(cookies) = self.cookies.as_deref() {
            let file = File::open(cookies)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;

                if line.starts_with('#') || line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() < 7 {
                    continue;
                }

                let domain = parts[0];
                let key = parts[5];
                let value = parts[6];

                let cookie_str = format!("{key}={value}; Domain={domain}");

                let url_str = format!("https://{}", domain.trim_start_matches('.'));
                let url = Url::parse(&url_str)?;

                jar.add_cookie_str(&cookie_str, &url);
            }
        }

        Ok(jar)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ExecutionMode {
    #[default]
    Parallel,
    Lazy,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
#[non_exhaustive]
pub enum Quality {
    #[value(name = "360")]
    P360 = 360,
    #[value(name = "480")]
    P480 = 480,
    #[default]
    #[value(name = "720")]
    P720 = 720,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[non_exhaustive]
pub enum TranslationTypeArg {
    Voice,
    Subtitles,
}

impl From<TranslationTypeArg> for TranslationType {
    fn from(arg: TranslationTypeArg) -> Self {
        match arg {
            TranslationTypeArg::Voice => Self::Voice,
            TranslationTypeArg::Subtitles => Self::Subtitles,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
#[non_exhaustive]
pub enum RelatedMode {
    #[default]
    None,
    #[value(name = "all")]
    All,
    #[value(name = "essential")]
    Essential,
}
