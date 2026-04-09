# kodik-parser

Library written in Rust for getting direct links to files from Kodik.

## Install
```sh
cargo add kodik-parser
```

## Usage

### Example
```rust
use kodik_parser::Client;

async fn main() {
    let client = Client::new();
    let url = "https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p";
    let kodik_response = kodik_parser::parse(&client, url).await.unwrap();

    let link_720 = &kodik_response.links.quality_720.first().unwrap().src;
    println!("Link with 720p quality is: {link_720}");
}
```
