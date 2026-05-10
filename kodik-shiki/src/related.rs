use crate::{FetchAnimesResponse, FetchAnimesVars, GraphQLRequest, Related};
use kodik_utils::{Client, Error, GET as _, POST as _};
use serde::Deserialize;

const LIMIT: usize = 50;

pub async fn fetch_related(client: &Client, franchise: &str, domain: &str) -> Result<Related, Error> {
    const ANIMES_BY_FRANCHISE_QUERY: &str = r"
query($franchise: String!, $page: PositiveInt!, $limit: PositiveInt!) {
  animes(franchise: $franchise, page: $page, limit: $limit, order: aired_on) {
    id
    name
    episodes

    related {
      relationKind
      anime {
        id
      }
    }

    userRate {
      status
      anime {
        name
      }
    }

    airedOn {
      date
    }
  }
}
";

    let graphql_url = format!("https://{domain}/api/graphql");
    let mut json = GraphQLRequest {
        query: ANIMES_BY_FRANCHISE_QUERY,
        variables: FetchAnimesVars::new(franchise),
    };

    let mut franchise = Related::default();
    for page in 1.. {
        json.variables.page = page;
        let mut resp: FetchAnimesResponse = client.post_json_as_json(&graphql_url, &json).await?;

        for index in (0..resp.data.animes.len()).rev() {
            if resp.data.animes[index].aired_on.date.is_none() {
                resp.data.animes.remove(index);
            }
        }

        let len = resp.data.animes.len();
        franchise.animes.append(&mut resp.data.animes);

        if len < LIMIT {
            break;
        }
    }

    Ok(franchise)
}

pub async fn fetch_not_anime_ids(client: &Client, neko_id: &str) -> Result<Option<Vec<usize>>, Error> {
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
        let franchise = "berserk";
        let domain = "shikimori.net";
        dbg!(fetch_related(&client, franchise, domain).await.unwrap());
    }
}
