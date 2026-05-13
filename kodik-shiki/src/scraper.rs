use std::fmt::Debug;

use kodik_utils::{Client, Error, GET};

use crate::KodikApiResponse;

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

/// Retrieves video results for an anime from Kodik.
///
/// # Errors
///
/// Returns `Error` if:
/// - The Kodik API request fails
pub async fn fetch_kodik_videos(client: &Client, shikimori_id: usize) -> Result<KodikApiResponse, Error> {
    let token = env!("KODIK_TOKEN");
    let url = format!(
        "https://kodik-api.com/search?token={token}&shikimori_id={shikimori_id}&with_seasons=true&with_episodes=true"
    );

    let search_response: KodikApiResponse = client.fetch_as_json(&url).await?;

    Ok(search_response)
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
