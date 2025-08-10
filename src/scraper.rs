use std::sync::LazyLock;

use reqwest::{Client, header::USER_AGENT};
use ua_generator::{fastrand, ua};

static AGENTS: LazyLock<&'static Vec<&'static str>> = LazyLock::new(ua::all_static_agents);

pub async fn get_with_fake_agent(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let agent = AGENTS[fastrand::usize(..AGENTS.len())];

    let response_text = client
        .get(url)
        .header(USER_AGENT, agent)
        .send()
        .await?
        .text()
        .await?;

    Ok(response_text)
}

pub async fn post_with_fake_user_agent(client: &Client, url: &str) {
    // let agent = AGENTS[fastrand::usize(..AGENTS.len())];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent() {
        let agent1 = AGENTS[fastrand::usize(..AGENTS.len())];
        let agent2 = AGENTS[fastrand::usize(..AGENTS.len())];
        assert_ne!(agent1, agent2);
    }

    #[tokio::test]
    async fn get() {
        let client = Client::new();
        let url = "https://kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let response = get_with_fake_agent(&client, url).await;
        assert!(response.is_ok());
    }
}
