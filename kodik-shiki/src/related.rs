use crate::{Anime, FetchAnimesResponse, FetchAnimesVars, GraphQLRequest, fetch_shiki_api_animes, parser};
use kodik_utils::{Client, Error, GET as _, POST as _};
use serde::Deserialize;

const LIMIT: usize = 50;

pub async fn fetch_franchise(client: &Client, url: &str) -> Result<Vec<Anime>, Error> {
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
    let franchise = fetch_shiki_api_animes(client, url).await?.franchise.unwrap();

    let mut accum = vec![];
    for page in 1.. {
        let json = GraphQLRequest {
            query: ANIMES_BY_FRANCHISE_QUERY,
            variables: FetchAnimesVars::new(&franchise, page),
        };
        dbg!(&json);

        let mut resp: FetchAnimesResponse = client.post_json_as_json(&graphql_url, &json).await?;

        let len = resp.data.animes.len();
        accum.append(&mut resp.data.animes);

        if len < LIMIT {
            break;
        }
    }

    Ok(accum)
}

pub async fn get_not_anime_ids(client: &Client, neko_id: &str) -> Result<Option<Vec<usize>>, Error> {
    type Achievements = Vec<Achievement>;

    #[derive(Deserialize)]
    struct Achievement {
        pub neko_id: String,
        pub level: Level,
        pub filters: Filters,
    }

    #[derive(Deserialize, PartialEq, Eq)]
    enum Level {
        #[serde(alias = "0")]
        Zero,
        #[serde(alias = "1")]
        One,
    }

    #[derive(Deserialize)]
    struct Filters {
        pub franchise: String,
        pub not_anime_ids: Option<Vec<usize>>,
    }

    let yaml_body = client.fetch_as_text("https://raw.githubusercontent.com/shikimori/neko-achievements/refs/heads/master/priv/rules/_franchises.yml").await?;

    let achievements: Achievements = serde_saphyr::from_str(&yaml_body)?;

    Ok(achievements
        .into_iter()
        .find(|ach| ach.level == Level::One && ach.neko_id == neko_id)
        .and_then(|ach| ach.filters.not_anime_ids))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_franchise_test() {
        let client = Client::new();
        let url = "https://shikimori.net/animes/33";
        dbg!(fetch_franchise(&client, url).await.unwrap());
    }
}
