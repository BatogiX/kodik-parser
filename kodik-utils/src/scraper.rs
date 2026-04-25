use reqwest::{
    Client, RequestBuilder, Response,
    header::{ACCEPT, COOKIE, HOST, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::{Error, random_user_agent};

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
pub fn build_headers(domain: &str, with_cookie: Option<&str>) -> Result<HeaderMap, crate::Error> {
    let mut headers = HeaderMap::with_capacity(if with_cookie.is_some() { 3 } else { 2 });
    headers.insert(HOST, HeaderValue::from_str(domain)?);

    if let Some(cookie) = with_cookie {
        let mut cookie_header = HeaderValue::from_str(cookie)?;
        cookie_header.set_sensitive(true);
        headers.insert(COOKIE, cookie_header);
    }

    Ok(headers)
}

/// Fetches data from the given URL and returns the response body as a string.
///
/// # Errors
///
/// Returns an [`Error`] if:
/// - A network request fails.
/// - The response body cannot be read as a string.
/// - An invalid URL is provided (though `reqwest` usually handles this during `get`).
pub async fn fetch_as_text(
    client: &Client,
    url: &str,
    headers: HeaderMap,
) -> Result<String, crate::Error> {
    log::info!("GET to {url}...");
    execute_text(client.get(url).headers(headers)).await
}

/// Fetches data from the given URL and deserializes it as JSON.
///
/// # Errors
///
/// Returns an [`Error`] if:
/// - A network request fails.
/// - The response cannot be deserialized into the target type `T`.
/// - An invalid URL is provided (though `reqwest` usually handles this during `get`).
pub async fn fetch_as_json<T: DeserializeOwned + Debug>(
    client: &Client,
    url: &str,
    headers: HeaderMap,
) -> Result<T, crate::Error> {
    log::info!("GET to {url}...");
    execute_json(client.get(url).headers(headers)).await
}

/// Posts data to the given URL and deserializes the response as JSON.
///
/// # Errors
///
/// Returns an [`Error`] if:
/// - A network request fails.
/// - The response cannot be deserialized into the target type `T`.
/// - An invalid URL is provided (though `reqwest` usually handles this during `post`).
pub async fn post_form_as_json<T, F>(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    form: &F,
) -> Result<T, crate::Error>
where
    T: DeserializeOwned + Debug,
    F: Serialize + Sync + ?Sized,
{
    log::info!("POST to {url}...");
    execute_json(client.post(url).form(form).headers(headers)).await
}

/// Posts JSON data to the given URL and deserializes the response as JSON.
///
/// # Errors
///
/// Returns an [`Error`] if:
/// - A network request fails.
/// - The response cannot be deserialized into the target type `T`.
/// - An invalid URL is provided (though `reqwest` usually handles this during `post`).
pub async fn post_json_as_json<T, J>(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    json: &J,
) -> Result<T, Error>
where
    T: DeserializeOwned + Debug,
    J: Serialize + Sync + ?Sized,
{
    log::info!("POST to {url}...");
    execute_json(client.post(url).json(json).headers(headers)).await
}

async fn execute_json<T>(builder: RequestBuilder) -> Result<T, crate::Error>
where
    T: DeserializeOwned + Debug,
{
    let resp = perform_request(builder.header(ACCEPT, "application/json")).await?;
    let data = resp.json::<T>().await?;
    log::trace!("Response data: {data:#?}");
    Ok(data)
}

async fn execute_text(builder: RequestBuilder) -> Result<String, crate::Error> {
    let resp = perform_request(builder).await?;
    let body = resp.text().await?;
    log::trace!("Response body: {body:#?}");
    Ok(body)
}

async fn perform_request(builder: RequestBuilder) -> Result<Response, crate::Error> {
    let resp = builder
        .header(USER_AGENT, random_user_agent())
        .send()
        .await?;

    Ok(resp)
}
