//! Stencila Cloud workspace assets API client
//!
//! Functions for uploading and fetching temporary workspace image assets.

use bytes::Bytes;
use eyre::{Result, eyre};
use reqwest::{
    Client,
    header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, EXPIRES, HeaderMap, LOCATION},
};
use serde::Deserialize;

use crate::{base_url, check_response, client, process_response, public_client};

/// Maximum asset size accepted by the Stencila Cloud API.
pub const MAX_ASSET_SIZE: usize = 10 * 1024 * 1024;

/// Supported asset content types.
pub const SUPPORTED_CONTENT_TYPES: [&str; 3] = ["image/png", "image/jpeg", "image/gif"];

/// Response from uploading a workspace asset.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetUploadResponse {
    /// Opaque URL-safe token for the asset.
    pub token: String,

    /// Public URL for fetching the asset while it is active.
    pub url: String,

    /// MIME type of the uploaded asset.
    pub content_type: String,

    /// Size of the uploaded asset in bytes.
    pub content_length: u64,

    /// ISO 8601 timestamp when the asset expires.
    pub expires_at: String,

    /// Location header returned by the upload endpoint.
    #[serde(default)]
    pub location: Option<String>,
}

/// Metadata returned by GET and HEAD asset responses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssetMetadata {
    /// MIME type of the asset.
    pub content_type: Option<String>,

    /// Size of the asset in bytes.
    pub content_length: Option<u64>,

    /// Cache policy for the asset response.
    pub cache_control: Option<String>,

    /// HTTP expiry time for the asset.
    pub expires: Option<String>,
}

/// Asset bytes and response metadata.
#[derive(Debug, Clone)]
pub struct AssetResponse {
    /// Raw image bytes.
    pub bytes: Bytes,

    /// Response metadata.
    pub metadata: AssetMetadata,
}

/// Upload a temporary image asset to a workspace.
///
/// The request body is sent as raw image bytes. The Cloud API currently accepts
/// PNG, JPEG, and GIF assets up to 10 MiB and returns a public URL that is
/// valid for one hour.
#[tracing::instrument(skip(bytes))]
pub async fn upload_asset(
    workspace_id: &str,
    content_type: &str,
    bytes: impl Into<Bytes>,
) -> Result<AssetUploadResponse> {
    let client = client().await?;
    upload_asset_with_client(&client, workspace_id, content_type, bytes).await
}

/// Upload a temporary image asset using an explicit Cloud API client.
#[tracing::instrument(skip(client, bytes))]
pub async fn upload_asset_with_client(
    client: &Client,
    workspace_id: &str,
    content_type: &str,
    bytes: impl Into<Bytes>,
) -> Result<AssetUploadResponse> {
    let bytes = bytes.into();
    let content_length = bytes.len();
    let url = format!("{}/workspaces/{workspace_id}/assets", base_url());

    tracing::debug!("Uploading asset to workspace {workspace_id}");

    let response = client
        .post(&url)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, content_length.to_string())
        .body(bytes)
        .send()
        .await?;

    if !response.status().is_success() {
        return process_response(response).await;
    }

    let location = optional_header(response.headers(), &LOCATION)?;
    let mut asset: AssetUploadResponse = process_response(response).await?;
    asset.location = location;

    Ok(asset)
}

/// Fetch a temporary workspace asset.
///
/// This endpoint does not require authentication while the token is active.
#[tracing::instrument]
pub async fn fetch_asset(workspace_id: &str, token: &str) -> Result<AssetResponse> {
    let url = asset_url(workspace_id, token);

    tracing::debug!("Fetching asset from {url}");

    let response = public_client()?.get(&url).send().await?;
    if !response.status().is_success() {
        check_response(response).await?;
        unreachable!("check_response should return an error for non-success responses")
    }

    let metadata = asset_metadata(response.headers())?;
    let bytes = response.bytes().await?;

    Ok(AssetResponse { bytes, metadata })
}

/// Check a temporary workspace asset.
///
/// This endpoint does not require authentication while the token is active.
#[tracing::instrument]
pub async fn check_asset(workspace_id: &str, token: &str) -> Result<AssetMetadata> {
    let url = asset_url(workspace_id, token);

    tracing::debug!("Checking asset at {url}");

    let response = public_client()?.head(&url).send().await?;
    if !response.status().is_success() {
        check_response(response).await?;
        unreachable!("check_response should return an error for non-success responses")
    }

    asset_metadata(response.headers())
}

/// Build the public URL for a temporary workspace asset.
pub fn asset_url(workspace_id: &str, token: &str) -> String {
    format!("{}/workspaces/{workspace_id}/assets/{token}", base_url())
}

fn asset_metadata(headers: &HeaderMap) -> Result<AssetMetadata> {
    Ok(AssetMetadata {
        content_type: optional_header(headers, &CONTENT_TYPE)?,
        content_length: optional_header(headers, &CONTENT_LENGTH)?
            .map(|value| value.parse())
            .transpose()
            .map_err(|error| eyre!("Invalid content-length header: {error}"))?,
        cache_control: optional_header(headers, &CACHE_CONTROL)?,
        expires: optional_header(headers, &EXPIRES)?,
    })
}

fn optional_header(
    headers: &HeaderMap,
    name: &reqwest::header::HeaderName,
) -> Result<Option<String>> {
    headers
        .get(name)
        .map(|value| value.to_str().map(String::from))
        .transpose()
        .map_err(|error| eyre!("Invalid response header: {error}"))
}
