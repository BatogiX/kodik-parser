use serde::{Deserialize, Serialize};

const LIMIT: usize = 50;

#[derive(Debug, Serialize)]
pub struct GraphQLRequest<V> {
    pub query: &'static str,
    pub variables: V,
}

#[derive(Debug, Serialize)]
pub struct FetchAnimesVars<'a> {
    pub franchise: &'a str,
    pub page: usize,
    pub limit: usize,
}

impl<'a> FetchAnimesVars<'a> {
    pub fn new(franchise: &'a str, page: usize) -> Self {
        Self {
            franchise,
            page,
            limit: LIMIT,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct FetchAnimesResponse {
    pub data: FetchAnimesData,
}

#[derive(Deserialize, Debug)]
pub struct FetchAnimesData {
    pub animes: Vec<Anime>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    pub id: String,
    pub name: String,
    pub episodes: usize,
    pub franchise: Option<String>,
    pub related: Vec<Relation>,
    pub user_rate: Option<UserRate>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub relation_kind: String,
    pub anime: Option<BasicAnime>,
}

#[derive(Deserialize, Debug)]
pub struct UserRate {
    pub status: String,
    pub anime: BasicAnime,
}

#[derive(Deserialize, Debug)]
pub struct BasicAnime {
    pub id: Option<String>,
    pub name: String,
}
