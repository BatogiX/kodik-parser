use kodik_parser::blocking;
use std::{env, process::exit};
use ureq::Agent;

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

    let agent = Agent::new_with_defaults();
    let kodik_response = match blocking::parse(&agent, url) {
        Ok(kodik_response) => kodik_response,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };

    let links = &kodik_response.links;
    if let Some(link) = links
        .quality_720
        .first()
        .or_else(|| links.quality_480.first())
        .or_else(|| links.quality_360.first())
    {
        print!("{}", link.src);
    } else {
        eprintln!("Error: no playable links found for this video");
        exit(1);
    }

    exit(0)
}
