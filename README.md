# kodik-parser

A Rust library for getting direct links to files from Kodik. 

## Features

- Automatic User-Agent substitution,

- Automatic search and caching of the current API endpoint,

- Automatic link decoding.

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
    let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let player_response = async_impl::parse(&client, url).await.unwrap();

    let link_720 = player_response.links.quality_720.first().unwrap().src;
    println!("Link with 720p quality is: {}", link_720);
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
    let url = "https://kodik.info/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let player_response = blocking::parse(&agent, url).unwrap();

    let link_720 = player_response.links.quality_720.first().unwrap().src;
    println!("Link with 720p quality is: {}", link_720);
}
```