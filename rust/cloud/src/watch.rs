//! Stencila Watches API client
//!
//! Functions for interacting with Stencila Cloud watches.
//! Watches sync files between repositories and external providers.
//! Watches are scoped under workspaces.

use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;

use crate::{base_url, check_response, client, process_response};

/// Request to create a watch for a document
///
/// Note: The repository is determined by the workspace context,
/// so repo_url is no longer needed in the request.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchRequest {
    /// The file path within the repository
    pub file_path: String,

    /// The remote URL (Google Docs, M365, or Stencila Site)
    pub remote_url: String,

    /// The sync direction: "bi", "from-remote", or "to-remote"
    pub direction: Option<String>,

    /// The PR mode: "draft" or "ready"
    pub pr_mode: Option<String>,

    /// The debounce time in seconds (10-86400)
    pub debounce_seconds: Option<u64>,
}

/// Response from creating a watch
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchResponse {
    /// The watch id (public ID like "wa7x2k9m3fab")
    pub id: String,
}

/// Create a watch for a document
///
/// This registers a watch with Stencila Cloud that will automatically sync
/// changes between a remote (Google Docs, M365, or Stencila Site) and a GitHub repository.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID (e.g., "ws3x9k2m7fab")
/// * `request` - The watch configuration
#[tracing::instrument]
pub async fn create_watch(workspace_id: &str, request: WatchRequest) -> Result<WatchResponse> {
    let client = client().await?;
    let url = format!("{}/workspaces/{}/watches", base_url(), workspace_id);

    tracing::debug!("Creating watch for file {} in workspace {}", request.file_path, workspace_id);
    let response = client.post(&url).json(&request).send().await?;

    process_response(response).await
}

/// Delete a watch
///
/// This removes a watch from Stencila Cloud, stopping automatic sync.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
/// * `watch_id` - The watch public ID
#[tracing::instrument]
pub async fn delete_watch(workspace_id: &str, watch_id: &str) -> Result<()> {
    let client = client().await?;
    let url = format!("{}/workspaces/{}/watches/{}", base_url(), workspace_id, watch_id);

    tracing::debug!("Deleting watch {watch_id} from workspace {workspace_id}");
    let response = client.delete(&url).send().await?;

    check_response(response).await
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
    pub workspace_id: Option<u64>,
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

/// Get a single watch by ID
///
/// This fetches detailed status information for a specific watch from Stencila Cloud.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
/// * `watch_id` - The watch public ID
#[tracing::instrument]
pub async fn get_watch(workspace_id: &str, watch_id: &str) -> Result<WatchDetailsResponse> {
    let client = client().await?;
    let url = format!("{}/workspaces/{}/watches/{}", base_url(), workspace_id, watch_id);

    tracing::debug!("Getting watch {watch_id} from workspace {workspace_id}");
    let response = client.get(&url).send().await?;

    process_response(response).await
}

/// Get all watches for a workspace
///
/// This fetches detailed status information for all watches in a workspace.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
#[tracing::instrument]
pub async fn get_watches(workspace_id: &str) -> Result<Vec<WatchDetailsResponse>> {
    let client = client().await?;
    let url = format!("{}/workspaces/{}/watches", base_url(), workspace_id);

    tracing::debug!("Listing watches for workspace {workspace_id}");
    let response = client.get(&url).send().await?;

    process_response(response).await
}
