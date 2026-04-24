use std::{collections::BTreeMap, fmt::Debug};

use kodik_utils::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::parser::{self, extract_id};

const LIMIT: usize = 50;

/// Retrieves video results for an anime from Kodik.
///
/// # Errors
///
/// Returns `KodikError` if:
/// - The domain cannot be extracted from the URL
/// - The anime ID cannot be extracted from the URL
/// - The Kodik API request fails
/// - No matching video source is found
pub async fn run(
    client: &Client,
    url: &str,
    cookie: Option<&str>,
    translation_title: Option<&str>,
    translation_type: Option<&TranslationType>,
    episode: Option<usize>,
) -> Result<VideoResult, Error> {
    let domain = kodik_utils::extract_domain(url)?;
    let id = extract_id(url)?;

    let search_response = get_kodik_videos(client, id).await?;

    let search_result =
        parser::find_search_result(search_response.results, translation_title, translation_type)?;

    if let Some(seasons) = search_result.seasons {
        let last_episode = if let Some(episode) = episode {
            episode
        } else if let Some(cookie) = cookie
            && let Ok(response) = kodik_utils::fetch_as_json::<SearchResponse>(
                client,
                &format!("https://{domain}/api/animes/{id}"),
                kodik_utils::build_headers(domain, Some(cookie))?,
            )
            .await
        // && let Some(user_rate) = response.user_rate
        {
            // user_rate.episodes
            1
        } else {
            0
        };

        let (_, season) = seasons
            .into_iter()
            .next_back()
            .ok_or(Error::NotFound("no season found".to_string()))?;

        let episodes = season
            .episodes
            .into_iter()
            .skip(last_episode)
            .map(|(_, ep)| ep)
            .collect();

        Ok(VideoResult::Episodes(episodes))
    } else {
        Ok(VideoResult::Film(search_result.link))
    }
}

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
    let url = format!(
        "https://kodik-api.com/search?token={token}&shikimori_id={id}&with_seasons=true&with_episodes=true"
    );

    let headers = kodik_utils::build_headers("kodik-api.com", None)?;
    kodik_utils::fetch_as_json(client, &url, headers).await
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchResult {
    pub link: String,
    pub translation: Translation,
    pub seasons: Option<BTreeMap<usize, Season>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Translation {
    pub title: String,
    pub r#type: TranslationType,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TranslationType {
    Voice,
    Subtitles,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Season {
    episodes: BTreeMap<usize, String>,
}

#[derive(Debug, Deserialize)]
pub struct FranchiseResponse {
    pub links: Vec<Link>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub source_id: usize,
    pub target_id: usize,
    pub relation: String,
}

#[derive(Debug, Deserialize)]
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

pub async fn get_not_anime_ids(
    client: &Client,
    neko_id: &str,
) -> Result<Option<Vec<usize>>, Error> {
    let yaml_body = kodik_utils::fetch_as_text(
        client,
        "https://raw.githubusercontent.com/shikimori/neko-achievements/refs/heads/master/priv/rules/_franchises.yml",
        kodik_utils::build_headers("raw.githubusercontent.com", None)?,
    ).await?;

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

pub async fn fetch_franchise(
    client: &Client,
    url: &str,
    ids: &str,
) -> Result<Option<String>, Error> {
    const FRANCHISE_QUERY: &str = "query($ids: String!) {
      animes(ids: $ids, limit: 1) {
        franchise
      }
    }";

    let json = GraphQLRequest {
        query: FRANCHISE_QUERY,
        variables: FetchFranchiseVars { ids },
    };

    let resp: FetchFranchiseResponse = kodik_utils::post_json_as_json(
        client,
        url,
        kodik_utils::build_headers(kodik_utils::extract_domain(url)?, None)?,
        &json,
    )
    .await?;

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
    with_cookie: Option<&str>,
) -> Result<FetchAnimesResponse, Error> {
    const ANIMES_BY_FRANCHISE_QUERY: &str =
        "query($franchise: String!, $page: PositiveInt!, $limit: PositiveInt!) {
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

    let resp: FetchAnimesResponse = kodik_utils::post_json_as_json(
        client,
        url,
        kodik_utils::build_headers(kodik_utils::extract_domain(url)?, with_cookie)?,
        &json,
    )
    .await?;

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
