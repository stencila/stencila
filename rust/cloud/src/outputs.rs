//! Stencila Cloud Outputs API client
//!
//! Functions for uploading files to Stencila Cloud workspace outputs.

use std::path::Path;

use eyre::{Result, eyre};
use reqwest::Client;
use serde::Deserialize;

use crate::{api_token, base_url, check_response, process_response};

/// Result of an upload attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadResult {
    /// File was uploaded (X-Upload-Performed header)
    Uploaded,
    /// File was unchanged and upload was skipped (X-Upload-Skipped header)
    Skipped,
}

/// Information about an uploaded output file
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
    /// File path relative to output root
    pub path: String,

    /// File size in bytes
    pub size: u64,

    /// Last modified timestamp (ISO 8601)
    pub last_modified: Option<String>,

    /// ETag/content hash
    pub etag: Option<String>,
}

/// Information about a git ref with output files
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputRef {
    /// Ref type: "branch", "tag", or "commit"
    pub ref_type: String,

    /// Ref name (branch name, tag name, or commit SHA)
    pub ref_name: String,

    /// Number of files for this ref
    pub file_count: usize,

    /// Total size of all files in bytes
    pub total_size: u64,

    /// Last modified timestamp (ISO 8601)
    pub last_modified: Option<String>,
}

/// Workspace outputs settings and statistics
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputsSettings {
    /// Whether outputs are enabled for this workspace
    pub enabled: bool,

    /// Total number of files across all refs
    pub total_files: usize,

    /// Total size of all files in bytes
    pub total_size: u64,
}

/// Normalize and encode file path for URL
///
/// - Convert backslashes to forward slashes (Windows compatibility)
/// - URL-encode special characters (spaces, unicode, etc.)
/// - Strip leading slashes
fn normalize_file_path(path: &str) -> String {
    // Normalize separators
    let normalized = path.replace('\\', "/");

    // Strip leading slashes
    let trimmed = normalized.trim_start_matches('/');

    // URL-encode each path segment separately (preserve /)
    trimmed
        .split('/')
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Upload a file to workspace outputs
///
/// Uploads a file to the specified workspace outputs location.
/// The cloud API uses content hashing to skip uploads of unchanged files.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
/// * `ref_type` - The ref type: "branch", "tag", or "commit"
/// * `ref_name` - The ref name (e.g., "main", "v1.0.0", commit SHA)
/// * `file_path` - The destination path within outputs (e.g., "report.pdf")
/// * `file` - Path to the local file to upload
///
/// # Returns
///
/// * `UploadResult::Uploaded` if the file was uploaded
/// * `UploadResult::Skipped` if the file was unchanged (based on content hash)
#[tracing::instrument]
pub async fn upload_output(
    workspace_id: &str,
    ref_type: &str,
    ref_name: &str,
    file_path: &str,
    file: &Path,
) -> Result<UploadResult> {
    let token = api_token()
        .ok_or_else(|| eyre!("Not authenticated. Run `stencila cloud signin` first."))?;

    let content = tokio::fs::read(file).await?;

    // Normalize and encode the file path
    let encoded_path = normalize_file_path(file_path);
    let encoded_ref = urlencoding::encode(ref_name);

    let url = format!(
        "{}/workspaces/{}/outputs/files/{}/{}/{}",
        base_url(),
        workspace_id,
        ref_type,
        encoded_ref,
        encoded_path
    );

    tracing::debug!("Uploading output file to {url}");

    let response = Client::new()
        .put(&url)
        .bearer_auth(token)
        .body(content)
        .send()
        .await?;

    // Check for errors before consuming the response
    if !response.status().is_success() {
        check_response(response).await?;
        // check_response should have returned an error
        unreachable!()
    }

    // Check upload status from headers
    if response.headers().get("X-Upload-Skipped").is_some() {
        Ok(UploadResult::Skipped)
    } else {
        Ok(UploadResult::Uploaded)
    }
}

/// List output files for a workspace
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
/// * `ref_type` - Optional ref type filter ("branch", "tag", "commit")
/// * `ref_name` - Optional ref name filter
///
/// # Returns
///
/// List of output files matching the filter criteria
#[tracing::instrument]
pub async fn list_output_files(
    workspace_id: &str,
    ref_type: Option<&str>,
    ref_name: Option<&str>,
) -> Result<Vec<OutputFile>> {
    let token = api_token()
        .ok_or_else(|| eyre!("Not authenticated. Run `stencila cloud signin` first."))?;

    let mut url = format!("{}/workspaces/{}/outputs/files", base_url(), workspace_id);

    // Add query parameters if provided
    let mut params = Vec::new();
    if let Some(rt) = ref_type {
        params.push(format!("refType={}", urlencoding::encode(rt)));
    }
    if let Some(rn) = ref_name {
        params.push(format!("refName={}", urlencoding::encode(rn)));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    tracing::debug!("Listing output files from {url}");

    let response = Client::new().get(&url).bearer_auth(token).send().await?;

    process_response(response).await
}

/// List refs with output files
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
///
/// # Returns
///
/// List of refs that have output files, with file counts and sizes
#[tracing::instrument]
pub async fn list_output_refs(workspace_id: &str) -> Result<Vec<OutputRef>> {
    let token = api_token()
        .ok_or_else(|| eyre!("Not authenticated. Run `stencila cloud signin` first."))?;

    let url = format!("{}/workspaces/{}/outputs/refs", base_url(), workspace_id);

    tracing::debug!("Listing output refs from {url}");

    let response = Client::new().get(&url).bearer_auth(token).send().await?;

    process_response(response).await
}

/// Get outputs settings and statistics
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
///
/// # Returns
///
/// Outputs settings including enabled status and usage statistics
#[tracing::instrument]
pub async fn get_outputs_settings(workspace_id: &str) -> Result<OutputsSettings> {
    let token = api_token()
        .ok_or_else(|| eyre!("Not authenticated. Run `stencila cloud signin` first."))?;

    let url = format!("{}/workspaces/{}/outputs", base_url(), workspace_id);

    tracing::debug!("Getting outputs settings from {url}");

    let response = Client::new().get(&url).bearer_auth(token).send().await?;

    process_response(response).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_file_path() {
        // Basic paths
        assert_eq!(normalize_file_path("report.pdf"), "report.pdf");
        assert_eq!(normalize_file_path("dir/file.pdf"), "dir/file.pdf");

        // Leading slashes stripped
        assert_eq!(normalize_file_path("/report.pdf"), "report.pdf");
        assert_eq!(normalize_file_path("///report.pdf"), "report.pdf");

        // Windows backslashes normalized
        assert_eq!(normalize_file_path("dir\\file.pdf"), "dir/file.pdf");
        assert_eq!(normalize_file_path("a\\b\\c\\file.pdf"), "a/b/c/file.pdf");

        // Spaces encoded
        assert_eq!(normalize_file_path("my file.pdf"), "my%20file.pdf");
        assert_eq!(
            normalize_file_path("my dir/my file.pdf"),
            "my%20dir/my%20file.pdf"
        );

        // Unicode encoded
        assert_eq!(normalize_file_path("café.pdf"), "caf%C3%A9.pdf");

        // Special characters encoded
        assert_eq!(normalize_file_path("file#1.pdf"), "file%231.pdf");
        assert_eq!(normalize_file_path("file?v=1.pdf"), "file%3Fv%3D1.pdf");
    }
}
