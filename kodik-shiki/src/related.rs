use kodik_utils::{Client, Error, GET as _};
use serde::Deserialize;
use tokio::sync::OnceCell;

const LIMIT: usize = 50;
static ACHIEVEMENTS: OnceCell<Vec<Achievement>> = OnceCell::const_new();

pub async fn fetch_not_anime_ids(client: &Client, neko_id: &str) -> Result<Option<&'static [usize]>, Error> {
    const ACHIEVEMENTS_URL: &str =
        "https://raw.githubusercontent.com/shikimori/neko-achievements/refs/heads/master/priv/rules/_franchises.yml";

    let achievements = ACHIEVEMENTS
        .get_or_try_init(|| async {
            let yaml_body = client.fetch_as_text(ACHIEVEMENTS_URL).await?;
            let achievements = serde_saphyr::from_str(&yaml_body)?;
            Ok::<Achievements, Error>(achievements)
        })
        .await?;

    Ok(achievements
        .iter()
        .find(|ach| ach.level == Level::One && ach.neko_id == neko_id)
        .and_then(|ach| ach.filters.not_anime_ids.as_deref()))
}

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    
}
