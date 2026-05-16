use reqwest::{
    Client, RequestBuilder, Response, StatusCode,
    header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Debug, future::Future, time::Duration};
use tokio::time;

use crate::{Error, ua};

pub trait POST {
    /// Posts data to the given URL and deserializes the response as JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A network request fails.
    /// - The response cannot be deserialized into the target type `T`.
    /// - An invalid URL is provided (though `reqwest` usually handles this during `post`).
    fn post_form_as_json<T, F>(&self, url: &str, form: &F) -> impl Future<Output = Result<T, crate::Error>> + Send
    where
        T: DeserializeOwned + Debug,
        F: Serialize + Sync + ?Sized;

    /// Posts JSON data to the given URL and deserializes the response as JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A network request fails.
    /// - The response cannot be deserialized into the target type `T`.
    /// - An invalid URL is provided (though `reqwest` usually handles this during `post`).
    fn post_json_as_json<T, J>(&self, url: &str, json: &J) -> impl Future<Output = Result<T, Error>> + Send
    where
        T: DeserializeOwned + Debug,
        J: Serialize + Sync + ?Sized;

    /// Posts JSON data to the given URL returns the response body as a string.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A network request fails.
    /// - An invalid URL is provided (though `reqwest` usually handles this during `post`).
    fn post_json_as_text<J>(&self, url: &str, json: &J) -> impl Future<Output = Result<String, Error>> + Send
    where
        J: Serialize + Sync + ?Sized;
}

pub trait GET {
    /// Fetches data from the given URL and returns the response body as a string.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A network request fails.
    /// - The response body cannot be read as a string.
    /// - An invalid URL is provided (though `reqwest` usually handles this during `get`).
    fn fetch_as_text(&self, url: &str) -> impl Future<Output = Result<String, crate::Error>> + Send;

    /// Fetches data from the given URL and deserializes it as JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A network request fails.
    /// - The response cannot be deserialized into the target type `T`.
    /// - An invalid URL is provided (though `reqwest` usually handles this during `get`).
    fn fetch_as_json<T: DeserializeOwned + Debug>(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<T, crate::Error>> + Send;
}

pub trait PATCH {
    fn patch_json_as_json<T, J>(&self, url: &str, json: &J) -> impl Future<Output = Result<T, Error>> + Send
    where
        T: DeserializeOwned + Debug,
        J: Serialize + Sync + ?Sized;
}

impl POST for Client {
    async fn post_form_as_json<T, F>(&self, url: &str, form: &F) -> Result<T, crate::Error>
    where
        T: DeserializeOwned + Debug,
        F: Serialize + Sync + ?Sized,
    {
        log::info!("POST to {url}...");
        execute_json(self.post(url).form(form)).await
    }

    async fn post_json_as_json<T, J>(&self, url: &str, json: &J) -> Result<T, Error>
    where
        T: DeserializeOwned + Debug,
        J: Serialize + Sync + ?Sized,
    {
        log::info!("POST to {url}...");
        execute_json(self.post(url).json(json)).await
    }

    async fn post_json_as_text<J>(&self, url: &str, json: &J) -> Result<String, Error>
    where
        J: Serialize + Sync + ?Sized,
    {
        log::info!("POST to {url}...");
        execute_text(self.post(url).json(json)).await
    }
}

impl GET for Client {
    async fn fetch_as_text(&self, url: &str) -> Result<String, crate::Error> {
        log::info!("GET to {url}...");
        execute_text(self.get(url)).await
    }

    async fn fetch_as_json<T: DeserializeOwned + Debug>(&self, url: &str) -> Result<T, crate::Error> {
        log::info!("GET to {url}...");
        execute_json(self.get(url)).await
    }
}

impl PATCH for Client {
    async fn patch_json_as_json<T, J>(&self, url: &str, json: &J) -> Result<T, Error>
    where
        T: DeserializeOwned + Debug,
        J: Serialize + Sync + ?Sized,
    {
        log::info!("PATCH to {url}...");
        execute_json(self.patch(url).json(json)).await
    }
}

/// Builds a `HeaderMap` with common headers.
///
/// # Arguments
///
/// * `host` - The value for the `Host` header.
/// * `with_cookie` - An optional string for the `Cookie` header. If `Some`, the cookie header will be marked as sensitive.
///
/// # Errors
///
/// Returns an [`Error`] if:
/// - The `host` string cannot be converted into a valid `HeaderValue`.
/// - The `with_cookie` string (if present) cannot be converted into a valid `HeaderValue`.
fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::with_capacity(7);

    headers.insert(USER_AGENT, HeaderValue::from_static(ua::random_user_agent()));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(HeaderName::from_static("dnt"), HeaderValue::from_static("1"));
    headers.insert(HeaderName::from_static("sec-gpc"), HeaderValue::from_static("1"));

    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );

    headers.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("document"),
    );

    headers.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("navigate"),
    );

    headers
}

async fn execute_json<T>(builder: RequestBuilder) -> Result<T, crate::Error>
where
    T: DeserializeOwned + Debug,
{
    let builder = builder.header(ACCEPT, "application/json");
    let resp = execute(builder).await?;
    let data = resp.json::<T>().await?;
    log::trace!("Response data: {data:#?}");
    Ok(data)
}

async fn execute_text(builder: RequestBuilder) -> Result<String, crate::Error> {
    let resp = execute(builder).await?;
    let body = resp.text().await?;
    log::trace!("Response body: {body:#?}");
    Ok(body)
}

async fn execute(builder: RequestBuilder) -> Result<Response, crate::Error> {
    const MAX_ATTEMPTS: u8 = 5;

    let headers = build_headers();
    let builder = builder.headers(headers);

    log::trace!("builder: {builder:#?}");

    for attempt in 1..=MAX_ATTEMPTS {
        let resp = builder.try_clone().expect("cannot clone builder").send().await?;

        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            let wait = Duration::from_secs((2_u64.pow(u32::from(attempt))).min(60));

            log::warn!("429 Too Many Requests. Waiting {wait:?} before retrying...");

            time::sleep(wait).await;
        } else {
            return Ok(resp);
        }
    }

    let resp = builder.send().await?;
    Ok(resp)
}
