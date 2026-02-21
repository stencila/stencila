//! Stencila Cloud database blob API client
//!
//! Functions for uploading and downloading content-addressed database blobs
//! (snapshots and changesets) used by the workspace database sync system.

use eyre::{Result, bail, eyre};
use reqwest::Client;

use crate::{api_token, base_url, check_response};
use stencila_version::STENCILA_USER_AGENT;

const BLOB_API_VERSION: &str = "v1";

fn blob_url(workspace_id: &str, kind: &str, hash: &str) -> String {
    format!(
        "{}/workspaces/{workspace_id}/db/{BLOB_API_VERSION}/{kind}/{hash}",
        base_url()
    )
}

/// Upload a content-addressed blob to Stencila Cloud
///
/// Blobs are immutable and keyed by their SHA-256 hash. Re-uploading an
/// existing hash is a no-op on the server side.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
/// * `kind` - Blob kind: `"snapshots"` or `"changesets"`
/// * `hash` - SHA-256 hex digest of `data`
/// * `data` - Raw blob bytes (consumed to avoid copying large snapshots)
#[tracing::instrument(skip(data))]
pub async fn upload_blob(
    workspace_id: &str,
    kind: &str,
    hash: &str,
    data: Vec<u8>,
) -> Result<()> {
    let token =
        api_token().ok_or_else(|| eyre!("Not authenticated. Run `stencila signin` first."))?;

    let url = blob_url(workspace_id, kind, hash);

    tracing::debug!("Uploading {kind} blob {hash} ({} bytes)", data.len());

    let response = Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .build()?
        .put(&url)
        .bearer_auth(token)
        .header("Content-Type", "application/octet-stream")
        .body(data)
        .send()
        .await?;

    check_response(response).await?;

    Ok(())
}

/// Download a content-addressed blob from Stencila Cloud
///
/// Verifies the downloaded content matches the expected hash.
///
/// # Arguments
///
/// * `workspace_id` - The workspace public ID
/// * `kind` - Blob kind: `"snapshots"` or `"changesets"`
/// * `hash` - Expected SHA-256 hex digest
#[tracing::instrument]
pub async fn download_blob(workspace_id: &str, kind: &str, hash: &str) -> Result<Vec<u8>> {
    let token =
        api_token().ok_or_else(|| eyre!("Not authenticated. Run `stencila signin` first."))?;

    let url = blob_url(workspace_id, kind, hash);

    tracing::debug!("Downloading {kind} blob {hash}");

    let response = Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .build()?
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        check_response(response).await?;
        unreachable!()
    }

    let bytes = response.bytes().await?;

    // Verify integrity
    use sha2::{Digest, Sha256};
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != hash {
        bail!("Blob integrity check failed: expected {hash}, got {actual}");
    }

    Ok(bytes.to_vec())
}
