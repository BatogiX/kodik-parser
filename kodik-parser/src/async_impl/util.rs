use std::sync::Arc;

use reqwest::Client;

use crate::{
    KodikError,
    async_impl::scraper,
    parser::{extract_player_url, get_api_endpoint},
    util,
};

pub async fn update_endpoint(
    client: &Client,
    domain: &str,
    response_text: &str,
) -> Result<(), KodikError> {
    let player_url = extract_player_url(domain, response_text)?;
    let player_response_text = scraper::get(client, &player_url).await?;
    let api_endpoint = get_api_endpoint(&player_response_text)?;
    util::set_endpoint(Arc::new(api_endpoint));

    Ok(())
}
