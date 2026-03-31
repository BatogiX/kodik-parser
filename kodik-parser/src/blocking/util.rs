use std::sync::Arc;

use arc_swap::Guard;
use ureq::Agent;

use crate::{
    KodikError,
    blocking::scraper,
    parser::{extract_endpoint, extract_player_url},
    state::KODIK_STATE,
};

/// Updates the API endpoint by extracting it from the player URL.
///
/// # Errors
///
/// Returns a `KodikError` if:
/// - The player URL cannot be extracted from the response
/// - The player response cannot be fetched
/// - The API endpoint cannot be extracted from the player response
pub fn update_endpoint(
    agent: &Agent,
    domain: &str,
    html: &str,
) -> Result<Guard<Arc<String>>, KodikError> {
    let player_url = extract_player_url(domain, html)?;
    let player_html = scraper::get(agent, &player_url)?;
    let endpoint = extract_endpoint(&player_html)?;
    KODIK_STATE.store_endpoint(Arc::new(endpoint));

    Ok(KODIK_STATE.load_endpoint())
}
