use std::sync::Arc;

use reqwest::Client;

use crate::{
    KodikError,
    async_impl::scraper,
    cache::KODIK_CACHE,
    parser::{extract_api_endpoint, extract_player_url},
};

pub async fn update_endpoint(
    client: &Client,
    domain: &str,
    response_text: &str,
) -> Result<(), KodikError> {
    let player_url = extract_player_url(domain, response_text)?;
    let player_response_text = scraper::get(client, &player_url).await?;
    let api_endpoint = extract_api_endpoint(&player_response_text)?;
    KODIK_CACHE.endpoint_store(Arc::new(api_endpoint));

    Ok(())
}
