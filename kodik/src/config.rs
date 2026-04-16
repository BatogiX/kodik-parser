use std::{collections::HashMap, fmt::Write, str::FromStr, sync::LazyLock};

use kodik_shiki::TranslationType;
use log::LevelFilter;

use crate::logging::{
    CYAN, CYAN_HIGH_INTENSITY_BOLD, GREEN_HIGH_INTENSITY_BOLD, RESET, YELLOW_BOLD,
};

pub enum ArgAction {
    /// Stores `true` when the flag is present.
    SetTrue,
    /// Stores the next token as a string value.
    Set,
    /// Increments a counter for each occurrence (`-v`, `-vv`).
    Count,
    /// Appends each token to a list (positional arguments).
    Append,
}

pub struct Arg {
    id: &'static str,
    short: Option<char>,
    long: Option<&'static str>,
    value_name: Option<&'static str>,
    help: &'static str,
    action: ArgAction,
}

impl Arg {
    #[must_use]
    pub const fn new(id: &'static str) -> Self {
        Self {
            id,
            short: None,
            long: None,
            value_name: None,
            help: "",
            action: ArgAction::SetTrue,
        }
    }

    #[must_use]
    pub const fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    #[must_use]
    pub const fn long(mut self, s: &'static str) -> Self {
        self.long = Some(s);
        self
    }

    #[must_use]
    pub const fn value_name(mut self, name: &'static str) -> Self {
        self.value_name = Some(name);
        self
    }

    #[must_use]
    pub const fn help(mut self, h: &'static str) -> Self {
        self.help = h;
        self
    }

    #[must_use]
    pub const fn action(mut self, action: ArgAction) -> Self {
        self.action = action;
        self
    }

    fn flags_display(&self) -> String {
        let base = match (self.short, self.long) {
            (Some(s), Some(l)) => format!("-{s}, --{l}"),
            (Some(s), None) => format!("-{s}"),
            (None, Some(l)) => format!("--{l}"),
            (None, None) => return String::new(),
        };
        match self.value_name {
            Some(v) => format!("{base} <{v}>"),
            None => base,
        }
    }
}

enum MatchValue {
    Flag(bool),
    Count(u8),
    Single(String),
    Multiple(Vec<String>),
}

pub struct ArgMatches {
    values: HashMap<&'static str, MatchValue>,
}

impl ArgMatches {
    #[must_use]
    pub fn get_flag(&self, id: &str) -> bool {
        matches!(self.values.get(id), Some(MatchValue::Flag(true)))
    }

    #[must_use]
    pub fn get_count(&self, id: &str) -> u8 {
        match self.values.get(id) {
            Some(MatchValue::Count(n)) => *n,
            _ => 0,
        }
    }

    #[must_use]
    pub fn get_one(&self, id: &str) -> Option<&str> {
        match self.values.get(id) {
            Some(MatchValue::Single(s)) => Some(s),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_many(&self, id: &str) -> &[String] {
        match self.values.get(id) {
            Some(MatchValue::Multiple(v)) => v,
            _ => &[],
        }
    }
}

pub struct Command {
    name: &'static str,
    args: Vec<Arg>,
}

impl Command {
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            args: Vec::new(),
        }
    }

    #[must_use]
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    #[must_use]
    pub fn help(&self) -> String {
        let named: Vec<&Arg> = self
            .args
            .iter()
            .filter(|a| !matches!(a.action, ArgAction::Append))
            .collect();
        let positional: Vec<&Arg> = self
            .args
            .iter()
            .filter(|a| matches!(a.action, ArgAction::Append))
            .collect();

        let width = named
            .iter()
            .map(|a| a.flags_display().len())
            .chain(
                positional
                    .iter()
                    .filter_map(|a| a.value_name)
                    .map(|v| v.len() + 5), // "[" + v + "]..."
            )
            .max()
            .unwrap_or(0);

        let pos_usage: String =
            positional
                .iter()
                .filter_map(|a| a.value_name)
                .fold(String::new(), |mut output, v| {
                    let _ = write!(output, " [{v}]...");
                    output
                });

        let mut s = format!(
            "{GREEN_HIGH_INTENSITY_BOLD}Usage:{RESET} {CYAN_HIGH_INTENSITY_BOLD}{}{RESET} \
             {CYAN}[OPTIONS]{pos_usage}{RESET}",
            self.name,
        );

        if !positional.is_empty() {
            let _ = write!(s, "\n\n{GREEN_HIGH_INTENSITY_BOLD}Arguments:{RESET}");
            for pos in &positional {
                if let Some(v) = pos.value_name {
                    let display = format!("[{v}]...");
                    let _ = write!(s, "\n  {CYAN}{display:<width$}{RESET}  {}", pos.help);
                }
            }
        }

        let _ = write!(s, "\n\n{GREEN_HIGH_INTENSITY_BOLD}Options:{RESET}");
        for arg in &named {
            let flags = arg.flags_display();
            let _ = write!(
                s,
                "\n  {CYAN_HIGH_INTENSITY_BOLD}{flags:<width$}{RESET}  {}",
                arg.help
            );
        }

        s
    }

    pub fn parse(&self, raw_args: Vec<String>) -> Result<ArgMatches, String> {
        let mut values: HashMap<&'static str, MatchValue> = HashMap::new();

        for arg in &self.args {
            match &arg.action {
                ArgAction::SetTrue => {
                    values.insert(arg.id, MatchValue::Flag(false));
                }
                ArgAction::Count => {
                    values.insert(arg.id, MatchValue::Count(0));
                }
                ArgAction::Append => {
                    values.insert(arg.id, MatchValue::Multiple(Vec::new()));
                }
                ArgAction::Set => {}
            }
        }

        let positional_id = self
            .args
            .iter()
            .find(|a| matches!(a.action, ArgAction::Append))
            .map(|a| a.id);

        let mut tokens = raw_args.into_iter().skip(1);

        while let Some(token) = tokens.next() {
            if token == "--" {
                for remaining in &mut tokens {
                    Self::push_positional(&mut values, positional_id, remaining);
                }
                break;
            }

            if let Some(long) = token.strip_prefix("--") {
                self.parse_long(&mut values, &mut tokens, long)?;
            } else if token.starts_with('-') && token.len() > 1 {
                self.parse_short(&mut values, &mut tokens, &token[1..])?;
            } else {
                Self::push_positional(&mut values, positional_id, token);
            }
        }

        Ok(ArgMatches { values })
    }

    fn push_positional(
        values: &mut HashMap<&'static str, MatchValue>,
        positional_id: Option<&'static str>,
        token: String,
    ) {
        if let Some(id) = positional_id
            && let Some(MatchValue::Multiple(list)) = values.get_mut(id)
        {
            list.push(token);
        }
    }

    fn parse_long(
        &self,
        values: &mut HashMap<&'static str, MatchValue>,
        tokens: &mut impl Iterator<Item = String>,
        long: &str,
    ) -> Result<(), String> {
        let arg = self
            .args
            .iter()
            .find(|a| a.long == Some(long))
            .ok_or_else(|| {
                format!(
                    "unexpected argument '{YELLOW_BOLD}--{long}{RESET}' found\n\n\
                     For more information, try '{CYAN_HIGH_INTENSITY_BOLD}--help{RESET}'.",
                )
            })?;

        match &arg.action {
            ArgAction::SetTrue => {
                values.insert(arg.id, MatchValue::Flag(true));
            }
            ArgAction::Count => {
                if let Some(MatchValue::Count(n)) = values.get_mut(arg.id) {
                    *n += 1;
                }
            }
            ArgAction::Set => {
                let val = tokens.next().ok_or_else(|| {
                    let v = arg.value_name.unwrap_or("VALUE");
                    format!(
                        "a value is required for \
                         '{YELLOW_BOLD}--{long} <{v}>{RESET}' but was not supplied",
                    )
                })?;
                values.insert(arg.id, MatchValue::Single(val));
            }
            ArgAction::Append => {}
        }

        Ok(())
    }

    fn parse_short(
        &self,
        values: &mut HashMap<&'static str, MatchValue>,
        tokens: &mut impl Iterator<Item = String>,
        cluster: &str,
    ) -> Result<(), String> {
        let chars: Vec<char> = cluster.chars().collect();
        let mut ci = 0;

        while ci < chars.len() {
            let c = chars
                .get(ci)
                .copied()
                .ok_or_else(|| "index out of bounds".to_string())?;

            let arg = self
                .args
                .iter()
                .find(|a| a.short == Some(c))
                .ok_or_else(|| {
                    format!(
                        "unexpected argument '{YELLOW_BOLD}-{c}{RESET}' found\n\n\
                         For more information, try '{CYAN_HIGH_INTENSITY_BOLD}--help{RESET}'.",
                    )
                })?;

            match &arg.action {
                ArgAction::SetTrue => {
                    values.insert(arg.id, MatchValue::Flag(true));
                    ci += 1;
                }
                ArgAction::Count => {
                    if let Some(MatchValue::Count(n)) = values.get_mut(arg.id) {
                        *n += 1;
                    }
                    ci += 1;
                }
                ArgAction::Set => {
                    let val = if ci + 1 < chars.len() {
                        chars
                            .get(ci + 1..)
                            .map(|slice| slice.iter().collect())
                            .ok_or_else(|| "index out of bounds".to_string())?
                    } else {
                        tokens.next().ok_or_else(|| {
                            let v = arg.value_name.unwrap_or("VALUE");
                            format!(
                                "a value is required for \
                                 '{YELLOW_BOLD}-{c} <{v}>{RESET}' but none was supplied",
                            )
                        })?
                    };
                    values.insert(arg.id, MatchValue::Single(val));
                    break;
                }
                ArgAction::Append => {
                    ci += 1;
                }
            }
        }

        Ok(())
    }
}

pub static COMMAND: LazyLock<Command> = LazyLock::new(|| {
    Command::new("kodik")
        .arg(
            Arg::new("url")
                .value_name("URL")
                .help("Url(s) to parse")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("lazy")
                .short('l')
                .long("lazy")
                .help("Outputs one by one (turns off parallelism)"),
        )
        .arg(
            Arg::new("player")
                .short('p')
                .long("player")
                .value_name("MEDIA-PLAYER")
                .help("Specify media player (implies --lazy)")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Use verbose output (-vv very verbose)")
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("silent")
                .short('s')
                .long("silent")
                .help("Do not print log messages"),
        )
        .arg(
            Arg::new("quality")
                .short('q')
                .long("quality")
                .value_name("QUALITY")
                .help("Specify video quality [possible values: 360, 480, 720] (default: 720)")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("episode")
                .short('e')
                .long("episode")
                .value_name("EPISODE")
                .help("Specify from which episode start with")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("translation_title")
                .long("title")
                .value_name("TITLE")
                .help("Specify translation title")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("translation_type")
                .long("type")
                .value_name("TYPE")
                .help("Specify translation type [possible values: voice, subtitles]")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("cookie")
                .long("cookie")
                .value_name("COOKIE")
                .help("Specify cookie to get your user rate")
                .action(ArgAction::Set),
        )
        .arg(Arg::new("help").short('h').long("help").help("Print help"))
});

#[derive(Debug, Clone, Copy, Default)]
pub enum Quality {
    P360 = 360,
    P480 = 480,
    #[default]
    P720 = 720,
}

impl FromStr for Quality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "360" => Ok(Self::P360),
            "480" => Ok(Self::P480),
            "720" => Ok(Self::P720),
            _ => Err(format!(
                "invalid value '{YELLOW_BOLD}{s}{RESET}' for '{CYAN_HIGH_INTENSITY_BOLD}-q{RESET}, {CYAN_HIGH_INTENSITY_BOLD}--quality <QUALITY>{RESET}'
  [possible values: {CYAN_HIGH_INTENSITY_BOLD}360{RESET}, {CYAN_HIGH_INTENSITY_BOLD}480{RESET}, {CYAN_HIGH_INTENSITY_BOLD}720{RESET}]\n
For more information, try '{CYAN_HIGH_INTENSITY_BOLD}--help{RESET}'."
            )),
        }
    }
}

pub struct TranslationTypeArg(pub Option<TranslationType>);

impl FromStr for TranslationTypeArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "voice" => Ok(Self(Some(TranslationType::Voice))),
            "subtitles" => Ok(Self(Some(TranslationType::Subtitles))),
            _ => Err(format!(
                "invalid value '{YELLOW_BOLD}{s}{RESET}' for '{CYAN_HIGH_INTENSITY_BOLD}--type <TYPE>{RESET}'
  [possible values: {CYAN_HIGH_INTENSITY_BOLD}voice{RESET}, {CYAN_HIGH_INTENSITY_BOLD}subtitles{RESET}]\n
For more information, try '{CYAN_HIGH_INTENSITY_BOLD}--help{RESET}'."
            )),
        }
    }
}

pub struct Config {
    pub urls: Vec<String>,
    pub level_filter: LevelFilter,
    pub lazy: bool,
    pub help: bool,
    pub player: Option<String>,
    pub quality: Quality,
    pub translation_title: Option<String>,
    pub translation_type: TranslationTypeArg,
    pub episode: Option<usize>,
    pub cookie: Option<String>,
}

impl Config {
    pub fn build(args: Vec<String>) -> Result<Self, String> {
        let m = COMMAND.parse(args)?;

        let level_filter = if m.get_flag("silent") {
            LevelFilter::Off
        } else {
            match m.get_count("verbose") {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            }
        };

        let quality = match m.get_one("quality") {
            Some(s) => s.parse()?,
            None => Quality::default(),
        };

        let translation_type = match m.get_one("translation_type") {
            Some(s) => s.parse::<TranslationTypeArg>()?,
            None => TranslationTypeArg(None),
        };

        let episode = m.get_one("episode").map(|src| {
            usize::from_str(src).map_err(|_| {
                format!(
                "invalid value '{YELLOW_BOLD}{src}{RESET}' for '{CYAN_HIGH_INTENSITY_BOLD}-e{RESET}, {CYAN_HIGH_INTENSITY_BOLD}--episode <EPISODE>{RESET}'\n
For more information, try '{CYAN_HIGH_INTENSITY_BOLD}--help{RESET}'."
                )
            })
        })
        .transpose()?;

        Ok(Self {
            urls: m.get_many("url").to_vec(),
            level_filter,
            lazy: m.get_flag("lazy"),
            help: m.get_flag("help"),
            player: m.get_one("player").map(str::to_owned),
            quality,
            translation_title: m.get_one("translation_title").map(str::to_owned),
            translation_type,
            episode,
            cookie: m.get_one("cookie").map(str::to_owned),
        })
    }
}
