use eyre::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::{ErrorResponse, base_url, client, process_response};

/// Request to create a watch for a document
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchRequest {
    /// The GitHub repository URL
    pub repo_url: String,

    /// The file path within the repository
    pub file_path: String,

    /// The remote URL (Google Docs or M365)
    pub remote_url: String,

    /// The sync direction
    pub direction: Option<String>,

    /// The PR mode (draft or ready)
    pub pr_mode: Option<String>,

    /// The debounce time in seconds
    pub debounce_seconds: Option<u64>,
}

/// Response from creating a watch
///
/// Note: other fields are available in response but are not currently necessary
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchResponse {
    /// The watch id
    pub id: u64,
}

/// Create a watch for a document
///
/// This registers a watch with Stencila Cloud that will automatically sync
/// changes between a remote (Google Docs or M365) and a GitHub repository.
pub async fn create_watch(request: WatchRequest) -> Result<WatchResponse> {
    let client = client().await?;
    let url = format!("{}/watches", base_url());

    let response = client.post(&url).json(&request).send().await?;

    process_response(response).await
}

/// Delete a watch
///
/// This removes a watch from Stencila Cloud, stopping automatic sync.
pub async fn delete_watch(watch_id: &str) -> Result<()> {
    let client = client().await?;
    let url = format!("{}/watches/{}", base_url(), watch_id);

    let response = client.delete(&url).send().await?;

    if !response.status().is_success() {
        let error_resp = response.json::<ErrorResponse>().await?;
        bail!("Failed to delete watch: {}", error_resp.error);
    }

    Ok(())
}
