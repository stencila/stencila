use eyre::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;

use crate::{ErrorResponse, base_url, client, process_response};

/// Request to create a watch for a document
#[skip_serializing_none]
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
    pub id: String,
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

/// Direction status for a watch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum WatchDirectionStatus {
    Ok,
    Pending,
    Running,
    Blocked,
    Error,
}

/// PR status for a watch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PrStatus {
    Open,
    Closed,
    Merged,
}

/// Full watch details response from the API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchDetailsResponse {
    pub id: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub user_id: Option<String>,
    pub org_id: Option<String>,
    pub repo_url: String,
    pub file_path: String,
    pub remote_url: String,
    pub provider: String,
    pub direction: String,
    pub pr_mode: String,
    pub debounce_seconds: u64,
    pub current_pr_number: Option<u64>,
    pub current_pr_status: Option<PrStatus>,
    pub last_remote_received_at: Option<String>,
    pub last_remote_processed_at: Option<String>,
    pub last_repo_received_at: Option<String>,
    pub last_repo_processed_at: Option<String>,
    pub last_repo_skipped_at: Option<String>,
    pub last_repo_skip_reason: Option<String>,
    pub from_remote_status: Option<WatchDirectionStatus>,
    pub to_remote_status: Option<WatchDirectionStatus>,
    pub last_remote_error: Option<String>,
    pub last_repo_error: Option<String>,
}

/// Get all watches for the authenticated user
///
/// This fetches detailed status information for all watches from Stencila Cloud.
/// Optionally filter by repository URL to reduce the response size.
pub async fn get_watches(repo_url: Option<&str>) -> Result<Vec<WatchDetailsResponse>> {
    let client = client().await?;
    let base = format!("{}/watches", base_url());

    let url = if let Some(repo) = repo_url {
        format!("{}?repoUrl={}", base, urlencoding::encode(repo))
    } else {
        base
    };

    let response = client.get(&url).send().await?;

    process_response(response).await
}
