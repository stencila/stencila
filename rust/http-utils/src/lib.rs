//! Convenience functions for making requests over HTTP
//!
//! This module provides a few functions that make it easier to make
//! requests over HTTP in a consistent manner e.g. with the 'User-Agent` header
//! set and respecting cache control headers in responses. In addition to reducing
//! the number of network requests for the client, several APIs ask clients
//! to implement caching to reduce load on their servers.

use std::io::Write;
use std::{env, fs::File, io, path::Path};

use eyre::Result;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tempfile::NamedTempFile;

// Re-exports for consumers of this crate
pub use reqwest;
pub use reqwest::header as headers;
pub use tempfile;

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

/// Get JSON from a URL
pub async fn get(url: &str) -> Result<serde_json::Value> {
    get_with(url, &[]).await
}

/// Get JSON from a URL with additional request headers
pub async fn get_with(url: &str, headers: &[(HeaderName, &str)]) -> Result<serde_json::Value> {
    let response = CLIENT
        .get(url)
        .headers(headers_to_map(
            &[&[(headers::ACCEPT, "application/json")], headers].concat(),
        )?)
        .send()
        .await?
        .error_for_status()?;

    let json = response.json().await?;

    Ok(json)
}

/// Download a file from a URL to a path
pub async fn download(url: &str, path: &Path) -> Result<()> {
    download_with(url, path, &[]).await
}

/// Download a file from a URL to a path with additional request headers
pub async fn download_with(url: &str, path: &Path, headers: &[(HeaderName, &str)]) -> Result<()> {
    let response = CLIENT
        .get(url)
        .headers(headers_to_map(headers)?)
        .send()
        .await?
        .error_for_status()?;

    let bytes = response.bytes().await?;
    let mut file = File::create(&path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;
    file.flush()?;

    Ok(())
}

/// Download a file from a URL to a path synchronously
pub fn download_sync(url: &str, path: &Path) -> Result<()> {
    download_with_sync(url, path, &[])
}

/// Download a file from a URL to a path synchronously with additional request headers
pub fn download_with_sync(url: &str, path: &Path, headers: &[(HeaderName, &str)]) -> Result<()> {
    let url = url.to_owned();
    let path = path.to_owned();
    let headers: Vec<(HeaderName, String)> = headers
        .iter()
        .map(|(header, value)| (header.clone(), value.to_string()))
        .collect();
    let (sender, receiver) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let headers: Vec<(HeaderName, &str)> = headers
            .iter()
            .map(|(header, value)| (header.clone(), value.as_str()))
            .collect();
        let result = download_with(&url, &path, &headers).await;
        sender.send(result)
    });
    receiver.recv()?
}

/// Download a file from a URL to a temporary file
///
/// Returns a `NamedTempFile` which will remove the temporary file when it
/// is dropped. Be aware of that and the security implications of long-lived temp files:
/// https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html
pub async fn download_temp(url: &str) -> Result<NamedTempFile> {
    let temp = NamedTempFile::new()?;
    download(url, temp.path()).await?;
    Ok(temp)
}

/// Convert an array of tuples to a `reqwest::HeaderMap`
fn headers_to_map(headers: &[(HeaderName, &str)]) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        header_map.insert(key, value.parse()?);
    }
    Ok(header_map)
}
