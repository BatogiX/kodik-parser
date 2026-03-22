mod cache;

use cache::KodikCache;
use kodik_parser::{Agent, KodikError};
use std::{env, process::exit, sync::Arc};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <url>", args[0]);
        exit(1);
    }
    if args.len() > 2 {
        eprintln!(
            "Error: unexpected argument '{}' found\n\nUsage: {} <url>",
            args[2], args[0]
        );
        exit(1);
    }
    let url = &args[1];

    let mut cache = KodikCache::load();
    cache.as_ref().map(KodikCache::apply_to_globals);
    let agent = Agent::new_with_defaults();

    let kodik_response = match (|| {
        let agent: &Agent = &agent;
        eprintln!("Extracting domain...");
        let domain = kodik_parser::parser::extract_domain(url)?;

        eprintln!("Fetching response...");
        let response_text = kodik_parser::blocking::scraper::get(agent, url)?;

        eprintln!("Extracting video info...");
        let video_info = kodik_parser::parser::extract_video_info(&response_text)?;
        let is_cached = !kodik_parser::util::get_endpoint().is_empty();
        if !is_cached {
            eprintln!("Endpoint not found in cache, updating...");
            kodik_parser::blocking::util::update_endpoint(agent, domain, &response_text)?;
        }
        let mut endpoint = kodik_parser::util::get_endpoint();

        eprintln!("Posting to endpoint...");
        let response_result =
            kodik_parser::blocking::scraper::post(agent, domain, &endpoint, &video_info);

        if let Ok(mut response) = response_result {
            eprintln!("Decoding links...");
            kodik_parser::decoder::decode_links(&mut response)?;
            Ok(response)
        } else if is_cached {
            eprintln!("Endpoint was deprecated in cache, updating... {endpoint}");
            kodik_parser::blocking::util::update_endpoint(agent, domain, &response_text)?;
            endpoint = kodik_parser::util::get_endpoint();
            eprintln!("Posting to endpoint... {endpoint}");
            let mut response =
                kodik_parser::blocking::scraper::post(agent, domain, &endpoint, &video_info)?;
            eprintln!("Decoding links...");
            kodik_parser::decoder::decode_links(&mut response)?;
            Ok(response)
        } else {
            response_result
        }
    })() {
        Ok(kodik_response) => kodik_response,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };

    if let Some(cache) = &mut cache
        && cache.is_changed()
    {
        eprintln!("Updating cache...");
        cache.update();
    }
    let links = &kodik_response.links;

    if let Some(link) = links
        .quality_720
        .first()
        .or_else(|| links.quality_480.first())
        .or_else(|| links.quality_360.first())
    {
        println!("{}", link.src);
    } else {
        eprintln!("Error: no playable links found for this video");
        exit(1);
    }

    exit(0)
}

fn update_endpoint(agent: &Agent, domain: &str, response_text: &str) -> Result<(), KodikError> {
    let player_url = kodik_parser::parser::extract_player_url(domain, response_text)?;
    let player_response_text = kodik_parser::blocking::scraper::get(agent, &player_url)?;
    let api_endpoint = kodik_parser::parser::get_api_endpoint(&player_response_text)?;
    kodik_parser::util::set_endpoint(Arc::new(api_endpoint));

    Ok(())
}
