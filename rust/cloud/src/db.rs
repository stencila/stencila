//! Stencila Cloud database blob API client
//!
//! Functions for uploading and downloading content-addressed database blobs
//! (snapshots and changesets) used by the workspace database sync system.

use std::sync::Arc;

use eyre::{Result, bail, eyre};
use reqwest::Client;
use serde::Deserialize;

use crate::{api_token, base_url, check_response};
use stencila_version::STENCILA_USER_AGENT;

/// Progress callback for blob downloads: `(bytes_received, total_bytes)`.
type ProgressCallback = dyn Fn(u64, Option<u64>) + Send + Sync;

const BLOB_API_VERSION: &str = "v1";

fn blob_url(workspace_id: &str, kind: &str, hash: &str) -> String {
    format!(
        "{}/workspaces/{workspace_id}/db/{BLOB_API_VERSION}/{kind}/{hash}",
        base_url()
    )
}

fn blob_list_url(workspace_id: &str, kind: &str) -> String {
    format!(
        "{}/workspaces/{workspace_id}/db/{BLOB_API_VERSION}/{kind}",
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
pub async fn upload_blob(workspace_id: &str, kind: &str, hash: &str, data: Vec<u8>) -> Result<()> {
    upload_blob_with_progress(workspace_id, kind, hash, data, None).await
}

/// Upload a content-addressed blob with an optional progress callback.
///
/// The callback receives the cumulative number of bytes sent so far.
#[tracing::instrument(skip(data, on_progress))]
pub async fn upload_blob_with_progress(
    workspace_id: &str,
    kind: &str,
    hash: &str,
    data: Vec<u8>,
    on_progress: Option<Arc<dyn Fn(u64) + Send + Sync>>,
) -> Result<()> {
    use futures::stream;

    let token =
        api_token().ok_or_else(|| eyre!("Not authenticated. Run `stencila signin` first."))?;

    let url = blob_url(workspace_id, kind, hash);
    let total_len = data.len();

    tracing::debug!("Uploading {kind} blob {hash} ({total_len} bytes)");

    const CHUNK_SIZE: usize = 64 * 1024; // 64 KB chunks

    let body = if let Some(cb) = on_progress {
        // Convert Vec<u8> into bytes::Bytes (zero-copy, takes ownership)
        // then slice into cheap reference-counted sub-views — no data
        // duplication regardless of blob size.
        let data = bytes::Bytes::from(data);
        let chunks: Vec<bytes::Bytes> = (0..total_len)
            .step_by(CHUNK_SIZE)
            .map(|start| {
                let end = (start + CHUNK_SIZE).min(total_len);
                data.slice(start..end)
            })
            .collect();

        let mut sent = 0u64;
        let stream = stream::iter(chunks.into_iter().map(move |chunk| {
            sent += chunk.len() as u64;
            cb(sent);
            Ok::<_, std::io::Error>(chunk)
        }));

        reqwest::Body::wrap_stream(stream)
    } else {
        reqwest::Body::from(data)
    };

    let response = Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .build()?
        .put(&url)
        .bearer_auth(token)
        .header("Content-Type", "application/octet-stream")
        .header("Content-Length", total_len.to_string())
        .body(body)
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
    download_blob_with_progress(workspace_id, kind, hash, None).await
}

/// Download a content-addressed blob with an optional progress callback.
///
/// The callback receives `(bytes_received, total_bytes)`. `total_bytes`
/// is `None` if the server did not send a `Content-Length` header.
#[tracing::instrument(skip(on_progress))]
pub async fn download_blob_with_progress(
    workspace_id: &str,
    kind: &str,
    hash: &str,
    on_progress: Option<Arc<ProgressCallback>>,
) -> Result<Vec<u8>> {
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

    let total = response.content_length();
    let mut received = 0u64;
    let mut buf = Vec::with_capacity(total.unwrap_or(0) as usize);

    let mut stream = response;
    while let Some(chunk) = stream.chunk().await? {
        received += chunk.len() as u64;
        buf.extend_from_slice(&chunk);
        if let Some(ref cb) = on_progress {
            cb(received, total);
        }
    }

    // Verify integrity
    use sha2::{Digest, Sha256};
    let actual = format!("{:x}", Sha256::digest(&buf));
    if actual != hash {
        bail!("Blob integrity check failed: expected {hash}, got {actual}");
    }

    Ok(buf)
}

#[derive(Debug, Deserialize)]
struct BlobListResponse {
    hashes: Vec<String>,
}

/// Default page size for paginated blob listing requests.
const BLOB_LIST_PAGE_SIZE: u64 = 100;

/// Maximum number of pages to fetch before bailing out.
/// Acts as a safeguard against non-terminating loops if the server
/// misbehaves (e.g. ignores the offset parameter).
const BLOB_LIST_MAX_PAGES: u64 = 1000;

/// List all blob hashes of a given kind stored on Stencila Cloud for a workspace.
///
/// The endpoint uses limit/offset pagination, so this function fetches pages
/// sequentially until an incomplete page signals the end of results.
#[tracing::instrument]
pub async fn list_blobs(workspace_id: &str, kind: &str) -> Result<Vec<String>> {
    let token =
        api_token().ok_or_else(|| eyre!("Not authenticated. Run `stencila signin` first."))?;

    let base_url = blob_list_url(workspace_id, kind);
    let client = Client::builder().user_agent(STENCILA_USER_AGENT).build()?;

    let mut all_hashes = Vec::new();
    let mut offset: u64 = 0;
    let mut page: u64 = 0;

    loop {
        page += 1;
        if page > BLOB_LIST_MAX_PAGES {
            bail!(
                "Blob listing exceeded {BLOB_LIST_MAX_PAGES} pages — \
                 possible server pagination issue"
            );
        }

        let url = format!("{base_url}?limit={BLOB_LIST_PAGE_SIZE}&offset={offset}");

        let response = client.get(&url).bearer_auth(&token).send().await?;

        if !response.status().is_success() {
            check_response(response).await?;
            unreachable!()
        }

        let page: BlobListResponse = response.json().await?;
        let count = page.hashes.len() as u64;
        all_hashes.extend(page.hashes);

        if count < BLOB_LIST_PAGE_SIZE {
            break;
        }

        offset += count;
    }

    Ok(all_hashes)
}

/// Delete a blob from Stencila Cloud.
#[tracing::instrument]
pub async fn delete_blob(workspace_id: &str, kind: &str, hash: &str) -> Result<()> {
    let token =
        api_token().ok_or_else(|| eyre!("Not authenticated. Run `stencila signin` first."))?;

    let url = blob_url(workspace_id, kind, hash);

    let response = Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .build()?
        .delete(&url)
        .bearer_auth(token)
        .send()
        .await?;

    check_response(response).await?;
    Ok(())
}
