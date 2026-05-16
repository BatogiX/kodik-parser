use crate::{Result, UserRate, models::shared::AnimeStatus};
use kodik_utils::{Client, POST as _};
use serde::{Deserialize, Serialize};
const LIMIT: usize = 50;
use crate::models::shared::deserialize_usize_from_string_or_number;

#[derive(Debug, Serialize)]
pub struct GraphQLRequest<V> {
    pub query: &'static str,
    pub variables: V,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchAnimesVars<'a> {
    pub franchise: &'a str,
    pub page: usize,
    pub limit: usize,
    pub exclude_ids: &'a str,
}

impl<'a> FetchAnimesVars<'a> {
    #[must_use]
    pub const fn new(franchise: &'a str, exclude_ids: &'a str) -> Self {
        Self {
            franchise,
            page: 1,
            limit: LIMIT,
            exclude_ids,
        }
    }
}

#[derive(Deserialize, Debug)]
struct FetchAnimesResponse {
    pub data: Related,
}

#[derive(Deserialize, Debug, Default)]
pub struct Related {
    pub animes: Vec<Anime>,
}

impl Related {
    pub async fn fetch_by_franchise(
        client: &Client,
        franchise: &str,
        domain: &str,
        not_anime_ids: &[usize],
    ) -> Result<Self> {
        const ANIMES_BY_FRANCHISE_QUERY: &str = r#"
    query($franchise: String!, $page: PositiveInt!, $limit: PositiveInt!, $excludeIds: String!) {
      animes(franchise: $franchise, page: $page, limit: $limit, excludeIds: $excludeIds, order: aired_on, status: "!anons") {
        id
        name
        status
        episodes
        episodesAired

        related {
          relationKind
          anime {
            id
          }
        }

        userRate {
          episodes
          id
          rewatches
          status
        }
      }
    }
    "#;

        let exclude_ids = not_anime_ids
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");

        let graphql_url = format!("https://{domain}/api/graphql");
        let mut json = GraphQLRequest {
            query: ANIMES_BY_FRANCHISE_QUERY,
            variables: FetchAnimesVars::new(franchise, &exclude_ids),
        };

        let mut related = Self::default();
        for page in 1.. {
            json.variables.page = page;
            let mut resp: FetchAnimesResponse = client.post_json_as_json(&graphql_url, &json).await?;
            let len = resp.data.animes.len();
            related.animes.append(&mut resp.data.animes);

            if len < LIMIT {
                break;
            }
        }

        Ok(related)
    }

    pub fn sort_by_chrono(&mut self) {
        self.animes.reverse();

        let mut current_index = 0;
        while current_index < self.animes.len() {
            let prequel_index = {
                let current_anime = &self.animes[current_index];

                current_anime
                    .related
                    .iter()
                    .filter(|relation| relation.relation_kind == RelationKind::Prequel)
                    .filter_map(|relation| {
                        self.animes
                            .iter()
                            .position(|anime| anime.id == relation.anime.as_ref().unwrap().id)
                    })
                    .find(|&index| index > current_index)
            };

            if let Some(prequel_index) = prequel_index {
                let prequel = self.animes.remove(prequel_index);
                self.animes.insert(current_index, prequel);

                self.animes[current_index + 1].related.clear();
            } else {
                self.animes[current_index].related.clear();
                current_index += 1;
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    #[serde(deserialize_with = "deserialize_usize_from_string_or_number")]
    pub id: usize,
    pub name: String,
    pub status: AnimeStatus,
    pub episodes: usize,
    pub episodes_aired: usize,
    pub related: Vec<Relation>,
    pub user_rate: Option<UserRate>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub relation_kind: RelationKind,
    pub anime: Option<BasicAnime>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Adaptation,
    AlternativeSetting,
    AlternativeVersion,
    Character,
    FullStory,
    Other,
    ParentStory,
    Prequel,
    Sequel,
    SideStory,
    SpinOff,
    Summary,
    Fan,
    Orig,
}

#[derive(Deserialize, Debug)]
pub struct BasicAnime {
    #[serde(deserialize_with = "deserialize_usize_from_string_or_number")]
    pub id: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_related_test() {
        let client = Client::new();
        let franchise = "berserk";
        let domain = "shikimori.io";
        dbg!(
            Related::fetch_by_franchise(&client, franchise, domain, &[])
                .await
                .unwrap()
        );
    }
}
