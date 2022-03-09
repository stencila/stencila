//! Convenience functions for making requests over HTTP
//!
//! This module provides a few functions that make it easier to make
//! requests over HTTP in a consistent manner e.g. with the 'User-Agent` header
//! set and respecting cache control headers in responses. In addition to reducing
//! the number of network requests for the client, several APIs ask clients
//! to implement caching to reduce load on their servers.

use std::fs::create_dir_all;
use std::io::Write;
use std::{env, fs::File, io, path::Path};

use eyre::Result;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tempfile::NamedTempFile;

// Re-exports for consumers of this crate
pub use reqwest;
pub use reqwest::header as headers;
pub use reqwest_middleware;
pub use serde;
pub use serde_json;
pub use tempfile;
pub use url;

pub static USER_AGENT: &str = concat!("stencila/", env!("CARGO_PKG_VERSION"),);

/// Get the directory of the HTTP cache
pub fn cache_dir() -> String {
    let user_cache_dir = dirs::cache_dir().unwrap_or_else(|| env::current_dir().unwrap());
    match env::consts::OS {
        "macos" | "windows" => user_cache_dir.join("Stencila").join("HTTP-Cache"),
        _ => user_cache_dir.join("stencila").join("http-cache"),
    }
    .to_string_lossy()
    .to_string()
}

pub static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Should be able to build HTTP client");
    let caching_middleware = Cache(HttpCache {
        mode: CacheMode::Default,
        manager: CACacheManager { path: cache_dir() },
        options: None,
    });
    ClientBuilder::new(client).with(caching_middleware).build()
});

/// Convert an array of tuples to a `reqwest::HeaderMap`
fn header_map(headers: &[(HeaderName, String)]) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    header_map.insert(headers::ACCEPT, HeaderValue::from_str("application/json")?);
    for (key, value) in headers {
        header_map.insert(key, value.parse()?);
    }
    Ok(header_map)
}

/// Send a GET request
pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T> {
    get_with(url, &[]).await
}

/// Send a GET request with additional request headers
pub async fn get_with<T: DeserializeOwned>(
    url: &str,
    headers: &[(HeaderName, String)],
) -> Result<T> {
    let response = CLIENT
        .get(url)
        .headers(header_map(headers)?)
        .send()
        .await?
        .error_for_status()?;

    let json = response.json().await?;

    Ok(json)
}

/// Send a POST request
pub async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: B) -> Result<T> {
    post_with(url, body, &[]).await
}

/// Send a POST request with additional request headers
pub async fn post_with<B: Serialize, T: DeserializeOwned>(
    url: &str,
    body: B,
    headers: &[(HeaderName, String)],
) -> Result<T> {
    let response = post_response(url, body, headers).await?;
    let json = response.json().await?;
    Ok(json)
}

/// Send a POST request with additional request headers and return response
pub async fn post_response<B: Serialize>(
    url: &str,
    body: B,
    headers: &[(HeaderName, String)],
) -> Result<Response> {
    let response = CLIENT
        .post(url)
        .json(&body)
        .headers(header_map(headers)?)
        .send()
        .await?
        .error_for_status()?;
    Ok(response)
}

/// Send a DELETE request
pub async fn delete(url: &str) -> Result<()> {
    delete_with(url, &[]).await
}

/// Send a DELETE request with additional request headers
pub async fn delete_with(url: &str, headers: &[(HeaderName, String)]) -> Result<()> {
    CLIENT
        .delete(url)
        .headers(header_map(headers)?)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Download a file from to a path
pub async fn download(url: &str, path: &Path) -> Result<()> {
    download_with(url, path, &[]).await
}

/// Download a file from to a path with additional request headers
pub async fn download_with(url: &str, path: &Path, headers: &[(HeaderName, String)]) -> Result<()> {
    let response = CLIENT
        .get(url)
        .headers(header_map(headers)?)
        .send()
        .await?
        .error_for_status()?;

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?
    }

    let bytes = response.bytes().await?;
    let mut file = File::create(&path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;
    file.flush()?;

    Ok(())
}

/// Download a file from to a path synchronously
pub fn download_sync(url: &str, path: &Path) -> Result<()> {
    download_with_sync(url, path, &[])
}

/// Download a file from to a path synchronously with additional request headers
pub fn download_with_sync(url: &str, path: &Path, headers: &[(HeaderName, String)]) -> Result<()> {
    let url = url.to_owned();
    let path = path.to_owned();
    let headers = headers.to_vec();
    let (sender, receiver) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let result = download_with(&url, &path, &headers).await;
        sender.send(result)
    });
    receiver.recv()?
}

/// Download a file from to a temporary file
pub async fn download_temp(url: &str, ext: Option<&str>) -> Result<NamedTempFile> {
    download_temp_with(url, ext, &[]).await
}

/// Download a file from to a temporary file with additional request headers
///
/// Returns a `NamedTempFile` which will remove the temporary file when it
/// is dropped. Be aware of that and the security implications of long-lived temp files:
/// https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html
pub async fn download_temp_with(
    url: &str,
    ext: Option<&str>,
    headers: &[(HeaderName, String)],
) -> Result<NamedTempFile> {
    let suffix = ext.map_or_else(String::new, |ext| [".", ext].concat());
    let temp = tempfile::Builder::new().suffix(&suffix).tempfile()?;
    download_with(url, temp.path(), headers).await?;
    Ok(temp)
}
