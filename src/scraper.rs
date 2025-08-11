use std::sync::LazyLock;

use reqwest::{Client, header::USER_AGENT};
use ua_generator::{fastrand, ua};

fn spoof_random_agent() -> &'static str {
    static AGENTS: LazyLock<&'static Vec<&'static str>> = LazyLock::new(ua::all_static_agents);
    AGENTS[fastrand::usize(..AGENTS.len())]
}


pub async fn get_with_fake_agent(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let agent = spoof_random_agent();

    let response_text = client.get(url).header(USER_AGENT, agent).send().await?.text().await?;

    Ok(response_text)
}

pub async fn post_with_fake_user_agent(client: &Client, url: &str) {
    let agent = spoof_random_agent();
}

pub async fn post_request_ti_kodik() {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_agent_is_not_always_same() {
        let a1 = spoof_random_agent();
        let a2 = spoof_random_agent();
        assert_ne!(a1, a2);
    }

    #[tokio::test]
    async fn get() {
        let client = Client::new();
        let url = "https://kodik.info/seria/1484069/6a2e103e9acf9829c6cba7e69555afb1/720p";
        let response = get_with_fake_agent(&client, url).await;
        assert!(response.is_ok());
    }
}
