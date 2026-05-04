use reqwest::{
    Client, RequestBuilder,
    header::{ACCEPT, HOST, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Debug, future::Future};

use crate::{Error, extract_domain, ua};

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

impl POST for Client {
    async fn post_form_as_json<T, F>(&self, url: &str, form: &F) -> Result<T, crate::Error>
    where
        T: DeserializeOwned + Debug,
        F: Serialize + Sync + ?Sized,
    {
        log::info!("POST to {url}...");
        let headers = build_headers(extract_domain(url)?)?;
        execute_json(self.post(url).form(form).headers(headers)).await
    }

    async fn post_json_as_json<T, J>(&self, url: &str, json: &J) -> Result<T, Error>
    where
        T: DeserializeOwned + Debug,
        J: Serialize + Sync + ?Sized,
    {
        log::info!("POST to {url}...");
        let headers = build_headers(extract_domain(url)?)?;
        execute_json(self.post(url).json(json).headers(headers)).await
    }
}
impl GET for Client {
    async fn fetch_as_text(&self, url: &str) -> Result<String, crate::Error> {
        log::info!("GET to {url}...");
        let headers = build_headers(extract_domain(url)?)?;
        execute_text(self.get(url).headers(headers)).await
    }

    async fn fetch_as_json<T: DeserializeOwned + Debug>(&self, url: &str) -> Result<T, crate::Error> {
        log::info!("GET to {url}...");
        let headers = build_headers(extract_domain(url)?)?;
        execute_json(self.get(url).headers(headers)).await
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
fn build_headers(domain: &str) -> Result<HeaderMap, crate::Error> {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(HOST, HeaderValue::from_str(domain)?);
    headers.insert(USER_AGENT, HeaderValue::from_str(ua::random_user_agent())?);

    Ok(headers)
}

async fn execute_json<T>(builder: RequestBuilder) -> Result<T, crate::Error>
where
    T: DeserializeOwned + Debug,
{
    let resp = builder.header(ACCEPT, "application/json").send().await?;
    let data = resp.json::<T>().await?;
    log::trace!("Response data: {data:#?}");
    Ok(data)
}

async fn execute_text(builder: RequestBuilder) -> Result<String, crate::Error> {
    let resp = builder.send().await?;
    let body = resp.text().await?;
    log::trace!("Response body: {body:#?}");
    Ok(body)
}
