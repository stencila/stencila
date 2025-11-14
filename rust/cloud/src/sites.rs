//! Stencila Sites API client
//!
//! Functions for interacting with Stencila Sites via the Cloud API.

use std::io::Write;
use std::{collections::HashMap, path::Path};

use chrono::DateTime;
use flate2::{Compression, write::GzEncoder};
use reqwest::header::LAST_MODIFIED;
use url::Url;

use eyre::{Result, bail, eyre};
use reqwest::Client;
use serde::Deserialize;
use tokio::fs::read;

use crate::{api_token, base_url};

/// Response from POST /sites
#[derive(Debug, Deserialize)]
struct CreateResponse {
    id: String,
}

/// Response from GET /sites/{siteId}/status
#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    /// Map of file path to file status
    pub files: HashMap<String, FileStatus>,
}

/// Status information for a single file in the site
#[derive(Debug, Clone, Deserialize)]
pub struct FileStatus {
    /// Last modified timestamp (Unix timestamp)
    pub modified_at: u64,

    /// ETag for cache validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// Create a new site
#[tracing::instrument]
pub async fn create_site() -> Result<String> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Creating Stencila Site");
    let client = Client::new();
    let response = client
        .post(format!("{}/sites", base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to create site ({status}): {error_text}");
    }

    let init_response: CreateResponse = response.json().await?;
    Ok(init_response.id)
}

/// Upload a single file to the site
#[tracing::instrument]
pub async fn upload_file(site_id: &str, path: &str, file: &Path) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    let url = format!("{}/sites/{}/latest/{}", base_url(), site_id, path);
    let content = read(file).await?;

    // TODO: determine whether to gzip content based on file extension
    // If gzipped then add a .gz to the path so that the serving worker
    // knows how to correctly set headers.

    tracing::debug!("Uploading to Stencila Site");
    let client = Client::new();
    let response = client
        .put(&url)
        .bearer_auth(token)
        .body(content)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to upload {path} ({status}): {error_text}");
    }

    Ok(())
}

/// Get the last modified time of a route on a Stencila Site
///
/// Makes a HEAD request to the URL (ensuring it has a trailing slash)
/// and returns the last-modified header as a Unix timestamp.
#[tracing::instrument]
pub async fn last_modified(url: &Url) -> Result<u64> {
    tracing::debug!("Fetching last-modified header from {url}");

    let client = Client::new();
    let response = client.head(url.to_string()).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        bail!("Failed to fetch ({status}): {url}");
    }

    // Extract the last-modified header
    let headers = response.headers();
    let last_modified = headers
        .get(LAST_MODIFIED)
        .ok_or_else(|| eyre!("No last-modified header found for {url}"))?;

    // Convert header value to string
    let last_modified_str = last_modified
        .to_str()
        .map_err(|e| eyre!("Invalid last-modified header value: {e}"))?;

    // Parse RFC 2822 timestamp and convert to Unix timestamp
    let datetime = DateTime::parse_from_rfc2822(last_modified_str)
        .map_err(|e| eyre!("Failed to parse last-modified header '{last_modified_str}': {e}"))?;

    let timestamp = datetime.timestamp() as u64;

    tracing::debug!("Last modified timestamp for {url}: {timestamp}");
    Ok(timestamp)
}

/// Compress content using gzip
fn gzip_content(content: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}
