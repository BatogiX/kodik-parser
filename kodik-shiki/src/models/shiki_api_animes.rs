use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ShikiApiAnimes {
    // id: usize,
    pub name: String,
    // russian: String,
    // url: String,
    // kind: String,
    // score: String,
    // status: String,
    // episodes: usize,
    // episodes_aired: usize,
    // aired_on: String,
    // released_on: String,
    // rating: String,
    pub franchise: Option<String>,
    pub user_rate: Option<UserRate>,
}

#[derive(Debug, Deserialize)]
pub struct UserRate {
    pub episodes: usize,
}
