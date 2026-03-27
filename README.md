# kodik-parser

A Rust library for getting direct links to files from Kodik.

## Features

- User-Agent substitution,

- Search and caching of the current API endpoint,

- Link decoding.

# Usage
## Async-impl
### Example
```
[dependencies.kodik-parser]
features = ["async-impl"]
```

```rust
use reqwest::Client;
use kodik_parser::async_impl;

async fn main() {
    let client = Client::new();
    let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let kodik_response = async_impl::parse(&client, url).await.unwrap();

    let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
    println!("Link with 720p quality is: {link_720}");
}
```

## Blocking
### Example
```
[dependencies.kodik-parser]
features = ["blocking"]
```

```rust
use ureq::Agent;
use kodik_parser::blocking;

fn main() {
    let agent = Agent::new_with_defaults();
    let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let kodik_response = blocking::parse(&agent, url).unwrap();

    let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
    println!("Link with 720p quality is: {link_720}");
}
```

## CLI
### Example
```sh
cargo build --bin kodik --features blocking --release 
```
```sh
./kodik
Usage: kodik <url>
```
```sh
./kodik https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p
https://p12.kodikplayer.com/s/m/aHR0cHM6Ly9jbG91ZC5rb2Rpay1zdG9yYWdlLmNvbS91c2VydXBsb2Fkcy8zOTkyYmZhOS05Yjc3LTQ4ZTItOGZjYS05ZGRmYTg5MzRhODU/491d50d71d07113553c74f1ceaa14677448d3848e9c57bf5fcc5d7ff936fe8b7:2026031721/720.mp4:hls:manifest.m3u8
```
