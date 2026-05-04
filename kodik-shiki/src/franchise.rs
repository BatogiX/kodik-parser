use kodik_utils::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{Response, parser};

const LIMIT: usize = 50;

pub async fn resolve_franchise(client: &Client, url: &str) -> Result<(), Error> {
    const ANIMES_BY_FRANCHISE_QUERY: &str = "query($franchise: String!, $page: PositiveInt!, $limit: PositiveInt!) {
  animes(franchise: $franchise, page: $page, limit: $limit) {
    id
    name
    episodes

    related {
      relationKind
      anime {
        id
        name
      }
    }

    userRate {
      status
      anime {
        name
      }
    }
  }
}
";

    let domain = kodik_utils::extract_domain(url)?;
    let graphql_url = format!("https://{domain}/api/graphql");
    let id = parser::extract_id(url)?;
    let franchise = fetch_franchise(client, domain, id).await?.unwrap();

    let mut accum = vec![];
    for page in 1.. {
        let json = GraphQLRequest {
            query: ANIMES_BY_FRANCHISE_QUERY,
            variables: FetchAnimesVars::new(&franchise, page),
        };
        dbg!(&json);

        let resp: FetchAnimesResponse = kodik_utils::post_json_as_json(
            client,
            &graphql_url,
            kodik_utils::build_headers(kodik_utils::extract_domain(url)?)?,
            &json,
        )
        .await?;

        let len = resp.data.animes.len();
        accum.push(resp.data.animes);

        if len < LIMIT {
            break;
        }
    }

    dbg!(accum);

    Ok(())
}

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
    fn new(franchise: &'a str, page: usize) -> Self {
        Self {
            franchise,
            page,
            limit: LIMIT,
        }
    }
}

async fn fetch_franchise(client: &Client, domain: &str, id: &str) -> Result<Option<String>, Error> {
    let shiki_resp = kodik_utils::fetch_as_json::<Response>(
        client,
        &format!("https://{domain}/api/animes/{id}"),
        kodik_utils::build_headers(domain)?,
    )
    .await?;

    Ok(shiki_resp.franchise)
}

#[derive(Deserialize, Debug)]
pub struct FetchAnimesResponse {
    pub data: FetchAnimesData,
}

#[derive(Deserialize, Debug)]
pub struct FetchAnimesData {
    pub animes: Vec<DetailedAnime>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetailedAnime {
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

pub async fn get_not_anime_ids(client: &Client, neko_id: &str) -> Result<Option<Vec<usize>>, Error> {
    let yaml_body = kodik_utils::fetch_as_text(
        client,
        "https://raw.githubusercontent.com/shikimori/neko-achievements/refs/heads/master/priv/rules/_franchises.yml",
        kodik_utils::build_headers("raw.githubusercontent.com")?,
    )
    .await?;

    let achievements: Achievements = serde_saphyr::from_str(&yaml_body)?;

    Ok(achievements
        .into_iter()
        .find(|ach| ach.level == Level::One && ach.neko_id == neko_id)
        .and_then(|ach| ach.filters.not_anime_ids))
}

type Achievements = Vec<Achievement>;

#[derive(Debug, Deserialize)]
struct Achievement {
    pub neko_id: String,
    pub level: Level,
    pub filters: Filters,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum Level {
    #[serde(alias = "0")]
    Zero,
    #[serde(alias = "1")]
    One,
}

#[derive(Debug, Deserialize)]
pub struct Filters {
    pub franchise: String,
    pub not_anime_ids: Option<Vec<usize>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_franchise_test() {
        let client = Client::new();
        let url = "https://shikimori.net/animes/33";
        resolve_franchise(&client, url).await.unwrap();
    }
}
