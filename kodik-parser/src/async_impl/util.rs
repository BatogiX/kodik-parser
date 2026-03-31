use std::sync::Arc;

use arc_swap::Guard;
use reqwest::Client;

use crate::{
    KodikError,
    async_impl::scraper,
    parser::{extract_endpoint, extract_player_url},
    state::KODIK_STATE,
};

pub async fn update_endpoint(
    client: &Client,
    domain: &str,
    html: &str,
) -> Result<Guard<Arc<String>>, KodikError> {
    let player_url = extract_player_url(domain, html)?;
    let player_html = scraper::get(client, &player_url).await?;
    let endpoint = extract_endpoint(&player_html)?;
    KODIK_STATE.store_endpoint(Arc::new(endpoint));

    Ok(KODIK_STATE.load_endpoint())
}
