//! Stencila Sites API client
//!
//! Functions for interacting with Stencila Sites via the Cloud API.

use std::io::Write;
use std::{collections::HashMap, path::Path};

use flate2::{Compression, write::GzEncoder};

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

/// Get the current status of files on the site from the Cloud API
#[tracing::instrument]
pub async fn get_site_status(site_id: &str) -> Result<Option<StatusResponse>> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Getting Stencila Site status");
    let client = Client::new();
    let response = client
        .get(format!("{}/sites/{}/status", base_url(), site_id))
        .bearer_auth(token)
        .send()
        .await?;

    if response.status().is_success() {
        let status = response.json::<StatusResponse>().await?;
        Ok(Some(status))
    } else if response.status().as_u16() == 404 {
        // Site doesn't have status yet (new site or API not implemented)
        tracing::info!("No site status available (new site or endpoint not implemented)");
        Ok(None)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to get site status ({status}): {error_text}");
    }
}

/// Compress content using gzip
fn gzip_content(content: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}
