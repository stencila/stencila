//! Stencila Sites API client
//!
//! Functions for interacting with Stencila Sites via the Cloud API.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use chrono::DateTime;
use clap::ValueEnum;
use eyre::{Result, bail, eyre};
use flate2::{Compression, write::GzEncoder};
use reqwest::Client;
use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tokio::fs::read;
use url::Url;

use stencila_config::{ConfigTarget, config, config_set, config_unset};

use crate::{api_token, base_url, check_response, process_response};

/// Access restriction mode for a site
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AccessMode {
    /// Anyone can view the site
    Public,
    /// Requires a password to view
    Password,
    /// Only authenticated team members can view
    Team,
}

impl AccessMode {
    /// Get the API value for this access mode
    pub fn api_value(&self) -> &'static str {
        match self {
            AccessMode::Public => "public",
            AccessMode::Password => "password",
            AccessMode::Team => "auth",
        }
    }
}

impl std::fmt::Display for AccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessMode::Public => write!(f, "public"),
            AccessMode::Password => write!(f, "password"),
            AccessMode::Team => write!(f, "team"),
        }
    }
}

/// Helper to get the site URL
///
/// Returns the custom domain URL if a domain is provided, otherwise returns
/// the default stencila.site subdomain URL.
pub fn default_site_url(site_id: &str, domain: Option<&str>) -> String {
    if let Some(domain) = domain {
        format!("https://{domain}")
    } else {
        format!("https://{}.stencila.site", site_id)
    }
}

/// Response from POST /sites
#[derive(Debug, Deserialize)]
struct CreateResponse {
    id: String,
}

/// Response from GET /sites/{id}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteDetails {
    pub id: String,
    pub domain: Option<String>,
    pub org_id: Option<String>,
    pub user_id: String,
    pub created_by: String,
    pub created_at: String,
    pub access_restriction: String,
    pub access_restrict_main: bool,
    pub access_updated_at: Option<String>,
}

/// Response from POST /sites/{id}/domain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainSetResponse {
    pub domain: String,
    pub status: String,
    pub cname_configured: Option<bool>,
    pub ssl_status: Option<String>,
    pub cname_record: String,
    pub cname_target: String,
    pub instructions: String,
}

/// Response from GET /sites/{id}/domain/status
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStatusResponse {
    pub configured: bool,
    pub domain: Option<String>,
    pub status: Option<String>,
    pub cname_configured: Option<bool>,
    pub ssl_status: Option<String>,
    pub cname_record: Option<String>,
    pub cname_target: Option<String>,
    pub error: Option<String>,
    pub message: String,
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

    let init_response: CreateResponse = process_response(response).await?;

    Ok(init_response.id)
}

/// Ensure a site configuration exists, creating it if necessary
///
/// Returns a tuple of (site_id, already_existed) where `already_existed` is true
/// if the site configuration was already present.
pub async fn ensure_site(path: &Path) -> Result<(String, bool)> {
    // Check if site config already exists
    let cfg = config(path)?;

    if let Some(site) = cfg.site
        && let Some(id) = site.id
    {
        return Ok((id, true));
    }

    // Need to create new configuration
    tracing::info!("No site id found, creating new Stencila Site");
    let id = create_site().await?;

    // Write to config
    config_set("site.id", &id, ConfigTarget::Nearest)?;

    Ok((id, false))
}

/// Get details for a site from Stencila Cloud
///
/// Fetches the site details including domain, ownership, access restrictions,
/// and timestamps.
#[tracing::instrument]
pub async fn get_site(site_id: &str) -> Result<SiteDetails> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Fetching site details for {site_id}");
    let client = Client::new();
    let response = client
        .get(format!("{}/sites/{site_id}", base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    process_response(response).await
}

/// Upload a single file to the site
#[tracing::instrument]
pub async fn upload_file(site_id: &str, branch_slug: &str, path: &str, file: &Path) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    let content = read(file).await?;

    // Compress HTML for faster uploads, smaller storage, and faster downloads
    let (path, body) = if file.extension().map(|ext| ext == "html").unwrap_or(false) {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&content)?;
        let compressed = encoder.finish()?;
        (format!("{path}.gz"), compressed)
    } else {
        (path.to_string(), content)
    };

    tracing::debug!("Uploading file to Stencila Site");
    let client = Client::new();
    let response = client
        .put(format!(
            "{}/sites/{site_id}/{branch_slug}/{path}",
            base_url()
        ))
        .bearer_auth(token)
        .body(body)
        .send()
        .await?;

    check_response(response).await
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

/// Get ETags for a list of storage paths on a site branch
///
/// Used for incremental pushes: compare local content hashes with server ETags
/// to skip uploading unchanged files.
///
/// # Arguments
///
/// * `site_id` - The site identifier
/// * `branch_slug` - The branch slug (e.g., "main", "feature-foo")
/// * `paths` - List of storage paths to get ETags for
///
/// # Returns
///
/// A map of storage path to ETag (quoted MD5 hex string like `"abc123..."`).
/// Paths that don't exist on the server are omitted from the response.
#[tracing::instrument]
pub async fn get_etags(
    site_id: &str,
    branch_slug: &str,
    paths: Vec<String>,
) -> Result<HashMap<String, String>> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Getting ETags for {} paths", paths.len());

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/sites/{site_id}/{branch_slug}/etags",
            base_url()
        ))
        .bearer_auth(token)
        .json(&paths)
        .send()
        .await?;

    process_response(response).await
}

/// Request to reconcile files at a prefix
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileRequest {
    /// List of file paths currently at this location
    pub paths: Vec<String>,

    /// The GitHub repository URL (for PR comments)
    pub repo_url: Option<String>,

    /// The branch name (for PR comments)
    pub branch_name: Option<String>,
}

/// Reconcile files at a prefix by cleaning up orphaned files
///
/// This function sends a list of current files to the Stencila Cloud API,
/// which will compare them with files in the bucket at the given prefix
/// and delete orphaned files.
///
/// If not on a default branch (main/master) and repo_url is provided,
/// the request includes PR comment info.
///
/// # Arguments
///
/// * `site_id` - The site identifier
/// * `repo_url` - The GitHub repository URL (empty string if not available)
/// * `branch_name` - The branch name (e.g., "main", "feature/foo")
/// * `branch_slug` - The branch slug (e.g., "main", "feature-foo")
/// * `prefix` - The storage path prefix (e.g., "media/", "report/media/")
/// * `current_files` - List of filenames (without prefix) currently at this location
#[tracing::instrument]
pub async fn reconcile_prefix(
    site_id: &str,
    repo_url: &str,
    branch_name: &str,
    branch_slug: &str,
    prefix: &str,
    current_files: Vec<String>,
) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!(
        "Reconciling prefix {prefix} with {} files",
        current_files.len()
    );

    // Include PR comment info only for non-default branches with a repo URL
    let (repo_url, branch_name) =
        if !repo_url.is_empty() && branch_slug != "main" && branch_slug != "master" {
            (Some(repo_url.to_string()), Some(branch_name.to_string()))
        } else {
            (None, None)
        };

    let request = ReconcileRequest {
        paths: current_files,
        repo_url,
        branch_name,
    };

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/sites/{site_id}/{branch_slug}/reconcile/{prefix}",
            base_url()
        ))
        .bearer_auth(token)
        .json(&request)
        .send()
        .await?;

    check_response(response).await
}

/// Delete a site from Stencila Cloud and remove local configuration
///
/// This function will:
/// 1. Read the site configuration to get the site ID
/// 2. Call DELETE /sites/{id} to remove the site from Stencila Cloud
/// 3. Remove the site.id from the local config
///
/// Returns the site ID that was deleted so that callers can perform additional
/// cleanup (e.g., removing remote tracking entries).
///
/// Note: This function does not prompt for user confirmation. Callers should
/// handle confirmation before calling this function.
#[tracing::instrument]
pub async fn delete_site(path: &Path) -> Result<String> {
    // Read existing site config to get the site ID
    let cfg = config(path)?;
    let site = cfg
        .site
        .ok_or_else(|| eyre!("No site configured for this workspace"))?;
    let site_id = site
        .id
        .ok_or_else(|| eyre!("Site ID not set in configuration"))?;

    // Get API token
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    // Call DELETE /sites/{id}
    tracing::debug!("Deleting Stencila Site");
    let client = Client::new();
    let response = client
        .delete(format!("{}/sites/{}", base_url(), site_id))
        .bearer_auth(token)
        .send()
        .await?;

    check_response(response).await?;

    // Remove from config
    config_unset("site.id", ConfigTarget::Nearest)?;

    tracing::debug!("Site deleted successfully");

    Ok(site_id)
}

/// Update site access settings
///
/// This function sends a PATCH request to `/sites/{site_id}/access` with
/// optional fields to update access restrictions, password, and main branch settings.
///
/// # Arguments
///
/// * `site_id` - The site identifier
/// * `access_mode` - Optional access mode to set
/// * `password` - Optional password to set (use Some(None) to clear password)
/// * `access_restrict_main` - Optional flag for whether main/master branches are restricted
#[tracing::instrument(skip(password))]
pub async fn update_site_access(
    site_id: &str,
    access_mode: Option<AccessMode>,
    password: Option<Option<&str>>,
    access_restrict_main: Option<bool>,
) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Updating access settings for site {site_id}");

    let mut json = serde_json::Map::new();

    if let Some(mode) = access_mode {
        json.insert(
            "accessRestriction".to_string(),
            serde_json::Value::String(mode.api_value().to_string()),
        );
    }

    if let Some(pwd) = password {
        json.insert(
            "password".to_string(),
            match pwd {
                Some(p) => serde_json::Value::String(p.to_string()),
                None => serde_json::Value::Null,
            },
        );
    }

    if let Some(restrict_main) = access_restrict_main {
        json.insert(
            "accessRestrictMain".to_string(),
            serde_json::Value::Bool(restrict_main),
        );
    }

    let client = Client::new();
    let response = client
        .patch(format!("{}/sites/{site_id}/access", base_url()))
        .bearer_auth(token)
        .json(&json)
        .send()
        .await?;

    check_response(response).await
}

/// Set password protection for a site
///
/// This function sends a PUT request to `/sites/{site_id}/password` with
/// the password and whether it should apply to the main branch.
///
/// # Arguments
///
/// * `site_id` - The site identifier
/// * `password` - The password to set
/// * `password_for_main` - Whether the password applies to the main branch (true by default)
#[tracing::instrument(skip(password))]
pub async fn set_site_password(
    site_id: &str,
    password: &str,
    password_for_main: bool,
) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Setting password for site {site_id}");

    let client = Client::new();
    let response = client
        .put(format!("{}/sites/{site_id}/password", base_url()))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "password": password,
            "accessRestrictMain": password_for_main
        }))
        .send()
        .await?;

    check_response(response).await
}

/// Remove password protection from a site
///
/// This function sends a DELETE request to `/sites/{site_id}/password`.
///
/// # Arguments
///
/// * `site_id` - The site identifier
#[tracing::instrument]
pub async fn remove_site_password(site_id: &str) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Removing password from site {site_id}");

    let client = Client::new();
    let response = client
        .delete(format!("{}/sites/{site_id}/password", base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    check_response(response).await
}

/// Set a custom domain for a site
///
/// This function sends a POST request to `/sites/{site_id}/domain` to configure
/// a custom domain. The API will return CNAME configuration instructions.
///
/// # Arguments
///
/// * `site_id` - The site identifier
/// * `domain` - The custom domain to set (e.g., "example.com")
#[tracing::instrument]
pub async fn set_site_domain(site_id: &str, domain: &str) -> Result<DomainSetResponse> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Setting domain for site {site_id}");

    let json = serde_json::json!({
        "domain": domain
    });

    let client = Client::new();
    let response = client
        .post(format!("{}/sites/{site_id}/domain", base_url()))
        .bearer_auth(token)
        .json(&json)
        .send()
        .await?;

    process_response(response).await
}

/// Get the status of a custom domain
///
/// This function sends a GET request to `/sites/{site_id}/domain/status` to check
/// the current status of domain configuration, CNAME setup, and SSL provisioning.
///
/// # Arguments
///
/// * `site_id` - The site identifier
#[tracing::instrument]
pub async fn get_site_domain_status(site_id: &str) -> Result<DomainStatusResponse> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Getting domain status for site {site_id}");

    let client = Client::new();
    let response = client
        .get(format!("{}/sites/{site_id}/domain/status", base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    process_response(response).await
}

/// Remove the custom domain from a site
///
/// This function sends a DELETE request to `/sites/{site_id}/domain` to remove
/// the custom domain configuration.
///
/// # Arguments
///
/// * `site_id` - The site identifier
#[tracing::instrument]
pub async fn delete_site_domain(site_id: &str) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Deleting domain for site {site_id}");

    let client = Client::new();
    let response = client
        .delete(format!("{}/sites/{site_id}/domain", base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    check_response(response).await
}

/// Information about a deployed branch
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    /// Branch name/slug
    pub name: String,

    /// Total number of files in the branch
    pub file_count: usize,

    /// Total size in bytes of all files
    pub total_size: u64,

    /// ISO 8601 timestamp of the most recently modified file
    pub last_modified: Option<String>,
}

/// List all deployed branches for a site
///
/// This function sends a GET request to `/sites/{site_id}/branches` to retrieve
/// information about all branches that have been deployed to the site.
///
/// # Arguments
///
/// * `site_id` - The site identifier
#[tracing::instrument]
pub async fn list_site_branches(site_id: &str) -> Result<Vec<BranchInfo>> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Listing branches for site {site_id}");

    let client = Client::new();
    let response = client
        .get(format!("{}/sites/{site_id}/branches", base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    process_response(response).await
}

/// Delete a branch from a site
///
/// This function sends a DELETE request to `/sites/{site_id}/branches/{branch_slug}`
/// to remove all files for a specific branch. The operation is asynchronous - a
/// workflow is triggered and files are deleted in the background.
///
/// Protected branches (main, master) cannot be deleted and will return an error.
///
/// # Arguments
///
/// * `site_id` - The site identifier
/// * `branch_slug` - The branch name to delete
#[tracing::instrument]
pub async fn delete_site_branch(site_id: &str, branch_slug: &str) -> Result<()> {
    let token = api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    tracing::debug!("Deleting branch {branch_slug} for site {site_id}");

    let client = Client::new();
    let response = client
        .delete(format!(
            "{}/sites/{site_id}/branches/{branch_slug}",
            base_url()
        ))
        .bearer_auth(token)
        .send()
        .await?;

    check_response(response).await
}
