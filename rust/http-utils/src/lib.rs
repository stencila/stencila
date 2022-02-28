//! Convenience functions for making requests over HTTP
//!
//! This module provides a few functions that make it easier to make
//! requests over HTTP in a consistent manner e.g. with the 'User-Agent` header
//! set and respecting cache control headers in responses. In addition to reducing
//! the number of network requests for the client, several APIs ask clients
//! to implement caching to reduce load on their servers.

use std::{env, fs::File, io, path::Path};

use eyre::Result;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use once_cell::sync::Lazy;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

static USER_AGENT: &str = concat!("stencila/", env!("CARGO_PKG_VERSION"),);

/// Get the directory of the HTTP cache
pub fn cache_dir() -> String {
    let user_cache_dir = dirs::cache_dir().unwrap_or_else(|| env::current_dir().unwrap());
    match env::consts::OS {
        "macos" | "windows" => user_cache_dir.join("Stencila").join("HTTP Cache"),
        _ => user_cache_dir.join("stencila").join("http-cache"),
    }
    .to_string_lossy()
    .to_string()
}

static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
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

// Get JSON from a URL
pub async fn get_json(url: &str) -> Result<serde_json::Value> {
    let response = CLIENT
        .get(url)
        .header("accept", "application/json")
        .send()
        .await?
        .error_for_status()?;

    let json = response.json().await?;

    Ok(json)
}

// Download a file from a URL to a path
pub async fn download_file(url: &str, path: &Path) -> Result<()> {
    let response = CLIENT.get(url).send().await?.error_for_status()?;

    let bytes = response.bytes().await?;
    let mut file = File::create(&path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;

    Ok(())
}

// Download a file from a URL to a path synchronously
pub fn download_file_sync(url: &str, path: &Path) -> Result<()> {
    let url = url.to_owned();
    let path = path.to_owned();
    let (sender, receiver) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let result = download_file(&url, &path).await;
        sender.send(result)
    });
    receiver.recv()?
}
