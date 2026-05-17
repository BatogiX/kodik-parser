use crate::cache::Cache;
use crate::config::{Config, Quality};
use anyhow::{self, Context as _, Result, bail};
use futures::future;
use reqwest::cookie::Jar;
use reqwest::{Client, Url};
use std::io::{self, BufWriter, Write as _};
use std::process::ExitCode;
use std::sync::Arc;

mod cache;
mod config;
mod logging;
mod shiki;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests;

pub async fn run(args: Vec<String>) -> ExitCode {
    match run_impl(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

async fn run_impl(args: Vec<String>) -> Result<()> {
    let config = Arc::new(Config::build(args).unwrap_or_else(|e| e.exit()));
    logging::setup_logging(config.level_filter());
    let mut cache = Cache::load();

    if let Some(ref cache) = cache {
        cache.apply();
    }

    let jar = Arc::new(config.load_cookies()?);
    let client = Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .gzip(true)
        .brotli(true)
        .zstd(true)
        .deflate(true)
        .build()?;

    let futures = config.urls.iter().map(|url| async {
        let client = &client;
        let config = &config;
        let urls = resolve_url(client, url, config, &jar).await?;

        let futures = urls.into_iter().map(|url| async move {
            let links = kodik_parser::parse(client, url.as_str()).await?;
            Ok::<String, anyhow::Error>(match config.quality {
                Quality::P360 => links.p360,
                Quality::P480 => links.p480,
                Quality::P720 => links.p720,
            })
        });

        let mut links = Vec::new();
        for link in future::join_all(futures).await {
            let link = link?;
            links.push(link);
        }

        Ok::<Vec<String>, anyhow::Error>(links)
    });

    let mut all_links = Vec::new();
    for result in future::join_all(futures).await {
        let mut links = result?;
        all_links.append(&mut links);
    }

    let mut stdout = BufWriter::new(io::stdout());
    for link in all_links {
        writeln!(stdout, "{link}")?;
    }
    stdout.flush()?;

    if let Some(ref mut cache) = cache
        && cache.is_changed()
    {
        log::warn!("updating cache... in {}", cache.path.display());
        cache.update();
        cache.save();
    }

    Ok(())
}

async fn resolve_url(client: &Client, url: &Url, config: &Config, jar: &Jar) -> Result<Vec<String>> {
    let links = match url
        .host_str()
        .with_context(|| format!("url '{url}' is not supported"))?
        .split_once('.')
        .with_context(|| format!("url '{url}' is not supported"))?
        .0
    {
        "shikimori" => shiki::resolve_shiki(client, url, config, jar).await?,
        "kodikplayer" => vec![url.to_string()],
        _ => bail!("url '{url}' is not supported"),
    };

    Ok(links)
}
