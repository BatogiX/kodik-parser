use crate::{Error, Result};
use kodik_utils::{Client, POST as _};
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
    #[must_use]
    pub const fn new(franchise: &'a str) -> Self {
        Self {
            franchise,
            page: 1,
            limit: LIMIT,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct FetchAnimesResponse {
    pub data: Related,
}

#[derive(Deserialize, Debug, Default)]
pub struct Related {
    pub animes: Vec<Anime>,
}

impl Related {
    pub async fn fetch_by_franchise(client: &Client, franchise: &str, domain: &str) -> Result<Self> {
        const ANIMES_BY_FRANCHISE_QUERY: &str = r#"
    query($franchise: String!, $page: PositiveInt!, $limit: PositiveInt!) {
      animes(franchise: $franchise, page: $page, limit: $limit, order: aired_on, status: "!anons") {
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
            id
          }
        }
      }
    }
    "#;

        let graphql_url = format!("https://{domain}/api/graphql");
        let mut json = GraphQLRequest {
            query: ANIMES_BY_FRANCHISE_QUERY,
            variables: FetchAnimesVars::new(franchise),
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

    #[must_use]
    pub fn filter_by_not_anime_ids(&mut self, not_anime_ids: &[usize]) -> Result<&mut Self> {
        for index in (0..self.animes.len()).rev() {
            let anime_id: usize = match self.animes[index].id.parse() {
                Ok(anime_id) => anime_id,
                Err(source) => {
                    return Err(Error::InvalidAnimeId {
                        value: self.animes[index].id.clone(),
                        source,
                    });
                }
            };

            if not_anime_ids.contains(&anime_id) {
                self.animes.remove(index);
            }
        }

        Ok(self)
    }

    #[must_use]
    pub fn sort_by_chrono(&mut self) -> &mut Self {
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

        self
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    pub id: String,
    pub name: String,
    pub episodes: usize,
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
pub struct UserRate {
    pub status: String,
    pub anime: BasicAnime,
    pub episodes: usize,
}

#[derive(Deserialize, Debug)]
pub struct BasicAnime {
    pub id: String,
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
        dbg!(Related::fetch_by_franchise(&client, franchise, domain).await.unwrap());
    }
}
