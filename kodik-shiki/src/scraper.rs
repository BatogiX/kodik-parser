use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use kodik_utils::{Error, GET, POST};
use lazy_regex::{Regex, regex};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum VideoResult {
    Episodes(Vec<String>),
    Film(String),
}

// pub async fn run_franchise(
//     client: &Client,
//     url: &str,
//     with_cookie: Option<&str>,
// ) -> Result<(), Error> {
//     // let ids = parser::extract_id(url)?;

//     // let franchise = fetch_franchise(client, url, ids)
//     //     .await?
//     //     .ok_or_else(|| Error::NotFound(format!("franchise not found for {url}")))?;

//     // let mut page = 1;
//     // let mut animes_vec: Vec<FetchAnimesResponse> = vec![];

//     // loop {
//     //     let animes = fetch_animes_by_franchise(client, url, &franchise, page, with_cookie).await?;
//     //     animes_vec.push(animes);

//     //     if animes.data.animes.len() < LIMIT {
//     //         break;
//     //     }
//     // }

//     // Ok(())
// }

pub async fn get_kodik_videos(client: &Client, id: &str) -> Result<SearchResponse, Error> {
    let token = env!("KODIK_TOKEN");
    let url =
        format!("https://kodik-api.com/search?token={token}&shikimori_id={id}&with_seasons=true&with_episodes=true");

    let search_response: SearchResponse = client.fetch_as_json(&url).await?;

    Ok(search_response)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchResponse {
    pub results: Results,
}

impl SearchResponse {
    pub fn find_search_result(
        &self,
        translation_title: Option<&str>,
        translation_type: Option<&TranslationType>,
    ) -> Result<&SearchResult, Error> {
        if let Some(translation_title) = translation_title {
            let title_re = Regex::new(&format!(r"(?i).*{}.*", regex::escape(translation_title)))?;

            if let Some(result) = self.results.iter().find(|r| title_re.is_match(&r.translation.title)) {
                log::info!("Found translation title '{}'", result.translation.title);
                return Ok(result);
            }

            log::warn!("no video source with title '{translation_title}'");
        } else if let Some(translation_type) = translation_type {
            if let Some(result) = self.results.iter().find(|r| r.translation.r#type == *translation_type) {
                log::info!("Found translation title '{}'", result.translation.title);
                return Ok(result);
            }

            log::warn!("no video source with type '{translation_type}'");
        }

        self.results
            .first()
            .ok_or_else(|| Error::NotFound("no video sources found".to_string()))
    }
}

type Results = Vec<SearchResult>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchResult {
    pub link: String,
    pub title: String,
    pub translation: Translation,
    pub seasons: Option<BTreeMap<usize, Season>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Translation {
    pub title: String,
    pub r#type: TranslationType,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationType {
    Voice,
    Subtitles,
}

impl Display for TranslationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Voice => write!(f, "voice"),
            Self::Subtitles => write!(f, "subtitles"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Season {
    pub episodes: BTreeMap<usize, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FranchiseResponse {
    pub links: Vec<Link>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Link {
    pub source_id: usize,
    pub target_id: usize,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Node {
    pub id: usize,
    pub name: String,
}

// pub async fn get_franchise(
//     client: &Client,
//     url: &str,
//     to_filter: bool,
// ) -> Result<Vec<String>, Error> {
//     let domain = extract_domain(url)?;
//     let id = extract_id(url)?;

//     let mut franchise: FranchiseResponse = kodik_utils::fetch_as_json(
//         client,
//         &format!("https://{domain}/api/animes/{id}/franchise"),
//         kodik_utils::build_headers(domain, None)?,
//     )
//     .await?;

//     if to_filter {
//         let response = get_response(client, domain, id, "").await?;
//         if let Some(neko_id) = response.franchise
//             && let Some(not_anime_ids) = get_not_anime_ids(client, &neko_id).await?
//         {
//             franchise
//                 .nodes
//                 .retain(|node| !not_anime_ids.contains(&node.id));
//         }
//     }

//     Ok(franchise
//         .nodes
//         .iter()
//         .rev()
//         .map(|node| format!("https://{domain}/animes/{}", node.id))
//         .collect())
// }

pub async fn fetch_franchise(client: &Client, url: &str, ids: &str) -> Result<Option<String>, Error> {
    const FRANCHISE_QUERY: &str = "query($ids: String!) {
      animes(ids: $ids, limit: 1) {
        franchise
      }
    }";

    let json = GraphQLRequest {
        query: FRANCHISE_QUERY,
        variables: FetchFranchiseVars { ids },
    };

    let resp: FetchFranchiseResponse = client.post_json_as_json(url, &json).await?;

    let first = resp.data.animes.into_iter().next();
    Ok(first.and_then(|a| a.franchise))
}

#[derive(Serialize)]
pub struct FetchFranchiseVars<'a> {
    pub ids: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct FetchFranchiseResponse {
    pub data: FetchFranchiseData,
}

#[derive(Deserialize, Debug)]
pub struct FetchFranchiseData {
    pub animes: Vec<FranchiseOnlyAnime>,
}

#[derive(Deserialize, Debug)]
pub struct FranchiseOnlyAnime {
    pub franchise: Option<String>,
}

pub async fn fetch_animes_by_franchise(
    client: &Client,
    url: &str,
    franchise: &str,
    page: usize,
) -> Result<FetchAnimesResponse, Error> {
    const LIMIT: usize = 50;
    const ANIMES_BY_FRANCHISE_QUERY: &str = "query($franchise: String!, $page: PositiveInt!, $limit: PositiveInt!) {
  animes(franchise: $franchise, page: $page, limit: $limit) {
    id
    name
    episodes
    franchise

    related {
      anime {
        id
        name
      }
      relationKind
    }

    userRate {
      anime {
        name
      }
      status
    }
  }
}
";

    let json = GraphQLRequest {
        query: ANIMES_BY_FRANCHISE_QUERY,
        variables: FetchAnimesVars {
            franchise,
            page,
            limit: LIMIT,
        },
    };

    let resp: FetchAnimesResponse = client.post_json_as_json(url, &json).await?;

    Ok(resp)
}

#[derive(Serialize)]
pub struct FetchAnimesVars<'a> {
    pub franchise: &'a str,
    pub page: usize,
    pub limit: usize,
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

#[derive(Serialize)]
pub struct GraphQLRequest<V> {
    pub query: &'static str,
    pub variables: V,
}
