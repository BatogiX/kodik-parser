use std::sync::Arc;

use ureq::Agent;

use crate::{
    KodikError,
    blocking::scraper,
    cache::KODIK_CACHE,
    parser::{extract_api_endpoint, extract_player_url},
};

/// Updates the API endpoint by extracting it from the player URL.
///
/// # Errors
///
/// Returns a `KodikError` if:
/// - The player URL cannot be extracted from the response
/// - The player response cannot be fetched
/// - The API endpoint cannot be extracted from the player response
pub fn update_endpoint(agent: &Agent, domain: &str, response_text: &str) -> Result<(), KodikError> {
    let player_url = extract_player_url(domain, response_text)?;
    let player_response_text = scraper::get(agent, &player_url)?;
    let api_endpoint = extract_api_endpoint(&player_response_text)?;
    KODIK_CACHE.endpoint_store(Arc::new(api_endpoint));

    Ok(())
}
