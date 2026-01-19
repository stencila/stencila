//! Stencila Workspaces API client
//!
//! Functions for interacting with Stencila Cloud workspaces.
//! Workspaces are the primary entity representing a GitHub repository.
//! Sites and watches are scoped under workspaces.

use std::path::Path;

use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

use stencila_config::{ConfigTarget, config, config_set};

use crate::{base_url, client, process_response};

/// Request to create a workspace
#[derive(Serialize)]
pub struct CreateWorkspaceRequest {
    /// The GitHub repository URL
    pub url: String,
}

/// Response from creating or getting a workspace
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceResponse {
    /// Internal database ID
    pub id: u64,

    /// Public ID (e.g., "ws3x9k2m7fab")
    pub public_id: String,

    /// User ID of the creator
    pub created_by: Option<String>,

    /// Username of the creator
    pub created_by_name: Option<String>,

    /// ISO 8601 creation timestamp
    pub created_at: Option<String>,

    /// User ID of the owner
    pub user_id: Option<String>,

    /// Organization ID if this is an org workspace
    pub org_id: Option<String>,

    /// Provider (e.g., "github")
    pub provider: Option<String>,

    /// Repository identifier (e.g., "owner/repo")
    pub identifier: Option<String>,

    /// Number of sessions created for this workspace
    pub sessions_count: Option<u64>,

    /// Timestamp of last session
    pub last_session_at: Option<String>,

    /// Whether email-to-PR is enabled
    pub email_enabled: Option<bool>,

    /// Email address for submissions (if enabled)
    pub email_address: Option<String>,

    /// User's permission level on the repository
    pub user_permission: Option<String>,

    /// Whether team access is enabled for the site
    pub site_team_access: Option<bool>,

    /// Whether access restrictions apply to main branch
    pub site_access_restrict_main: Option<bool>,

    /// Custom domain for the site
    pub site_domain: Option<String>,

    /// Custom domain status
    pub site_domain_status: Option<String>,

    /// Custom domain error message
    pub site_domain_error: Option<String>,

    /// Whether a site password is set
    pub site_password_set: Option<bool>,
}

/// Create or get a workspace for a GitHub repository
///
/// The Stencila Cloud API will return an existing workspace if one already
/// exists for the same repository/organization combination.
///
/// # Arguments
///
/// * `github_url` - The GitHub repository URL (e.g., "https://github.com/owner/repo")
///
/// # Returns
///
/// The workspace response containing the public_id and other details.
#[tracing::instrument]
pub async fn create_or_get_workspace(github_url: &str) -> Result<WorkspaceResponse> {
    let client = client().await?;
    let url = format!("{}/workspaces", base_url());

    let request = CreateWorkspaceRequest {
        url: github_url.to_string(),
    };

    tracing::debug!("Creating or getting workspace for {github_url}");
    let response = client.post(&url).json(&request).send().await?;

    process_response(response).await
}

/// Get a workspace by its public ID
///
/// # Arguments
///
/// * `workspace_id` - The workspace's public ID (e.g., "ws3x9k2m7fab")
#[tracing::instrument]
pub async fn get_workspace(workspace_id: &str) -> Result<WorkspaceResponse> {
    let client = client().await?;
    let url = format!("{}/workspaces/{}", base_url(), workspace_id);

    tracing::debug!("Getting workspace {workspace_id}");
    let response = client.get(&url).send().await?;

    process_response(response).await
}

/// List workspaces accessible to the authenticated user
#[tracing::instrument]
pub async fn list_workspaces() -> Result<Vec<WorkspaceResponse>> {
    let client = client().await?;
    let url = format!("{}/workspaces", base_url());

    tracing::debug!("Listing workspaces");
    let response = client.get(&url).send().await?;

    process_response(response).await
}

/// Ensure a workspace exists for the current repository
///
/// This function:
/// 1. Checks if workspace.id is already in config
/// 2. If not, derives GitHub URL from git remote and calls POST /v1/workspaces
/// 3. Stores the workspace public_id in config
///
/// # Arguments
///
/// * `path` - Path to a file or directory in the repository
///
/// # Returns
///
/// A tuple of (workspace_id, already_existed) where `already_existed` is true
/// if the workspace configuration was already present.
#[tracing::instrument]
pub async fn ensure_workspace(path: &Path) -> Result<(String, bool)> {
    // Check if workspace config already exists
    let cfg = config(path)?;

    if let Some(workspace) = cfg.workspace
        && let Some(id) = workspace.id
    {
        tracing::debug!("Workspace already configured: {id}");
        return Ok((id, true));
    }

    // Check for STENCILA_WORKSPACE_ID environment variable (set in sync sessions)
    if let Ok(id) = std::env::var("STENCILA_WORKSPACE_ID")
        && !id.is_empty()
    {
        tracing::debug!("Workspace ID from STENCILA_WORKSPACE_ID env var: {id}");
        return Ok((id, true));
    }

    // Need to create - derive GitHub URL from git remote
    let git_repo_info = stencila_codec_utils::git_repo_info(path)?;
    let github_url = git_repo_info.origin.ok_or_else(|| {
        eyre!("No git origin remote found. A git repository with an origin is required to create a workspace.")
    })?;

    tracing::info!("No workspace.id found, creating workspace for {github_url}");

    // Create or get workspace
    let workspace = create_or_get_workspace(&github_url).await?;

    // Write to config
    config_set("workspace.id", &workspace.public_id, ConfigTarget::Nearest)?;

    tracing::info!("Created workspace with ID: {}", workspace.public_id);

    Ok((workspace.public_id, false))
}
