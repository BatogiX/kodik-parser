use crate::{Error, Result};
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
    #[must_use]
    pub fn filter_by_not_anime_ids(&mut self, not_anime_ids: &[usize]) -> Result<()> {
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

        Ok(())
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
    pub id: String,
    pub name: String,
    pub episodes: usize,
    pub related: Vec<Relation>,
    pub user_rate: Option<UserRate>,
    pub aired_on: AiredOn,
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
}

#[derive(Deserialize, Debug)]
pub struct BasicAnime {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct AiredOn {
    pub date: Option<String>,
}
