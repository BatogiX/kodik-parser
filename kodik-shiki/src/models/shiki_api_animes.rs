use serde::Deserialize;

use crate::{UserRate, models::shared::AnimeStatus};

#[derive(Debug, Deserialize)]
pub struct ShikiApiAnimes {
    pub id: usize,
    pub name: String,
    // russian: String,
    // url: String,
    // kind: String,
    // score: String,
    pub status: AnimeStatus,
    pub episodes: usize,
    pub episodes_aired: usize,
    // aired_on: String,
    // released_on: String,
    // rating: String,
    pub franchise: Option<String>,
    pub user_rate: Option<UserRate>,
}
