//! Stencila Cloud workspace links API client
//!
//! Functions for creating and reading JSON snapshots addressed by
//! `https://stencila.link` URLs.

use eyre::{Result, bail, eyre};
use reqwest::{
    StatusCode,
    header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, HeaderMap, LOCATION},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{base_url, check_response, client, process_response};

/// The canonical base URL for Stencila links.
pub const LINK_URL_BASE: &str = "https://stencila.link";

/// Maximum snapshot body size accepted by the Stencila Cloud API.
pub const MAX_SNAPSHOT_SIZE: usize = 25 * 1024 * 1024;

/// Optional content encoding for snapshot request bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotContentEncoding {
    /// Gzip-compressed JSON body.
    Gzip,
}

impl SnapshotContentEncoding {
    fn as_str(self) -> &'static str {
        match self {
            SnapshotContentEncoding::Gzip => "gzip",
        }
    }
}

/// Canonical JSON envelope returned by the Links API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotEnvelope {
    /// Snapshot envelope format version.
    pub version: u8,

    /// Stencila JSON node.
    pub node: Value,

    /// Optional parent Stencila JSON node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Value>,
}

impl SnapshotEnvelope {
    /// Create a version 1 snapshot envelope.
    pub fn new(node: Value, parent: Option<Value>) -> Self {
        Self {
            version: 1,
            node,
            parent,
        }
    }
}

/// Response from writing a workspace link snapshot.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LinkSnapshotResponse {
    /// Link public ID.
    pub id: String,

    /// Canonical lowercase SHA-256 of the snapshot.
    pub sha256: String,

    /// Canonical link URL.
    pub url: String,

    /// `Location` header returned by the API.
    #[serde(default)]
    pub location: Option<String>,
}

/// Metadata returned when checking an existing snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkSnapshotMetadata {
    /// Canonical link URL from the `Location` header, if present.
    pub location: Option<String>,
}

/// Build the canonical link URL for a public link ID.
pub fn link_url(public_id: &str) -> String {
    format!("{LINK_URL_BASE}/{public_id}")
}

/// Build the JSON snapshot URL for a public link ID.
pub fn link_json_url(public_id: &str) -> String {
    format!("{}/{public_id}/json", LINK_URL_BASE)
}

/// Write a JSON snapshot to a workspace and create a link.
///
/// The body may be either a bare Stencila JSON node or a [`SnapshotEnvelope`].
/// This uses the configured Stencila API key as a bearer token.
#[tracing::instrument(skip(snapshot))]
pub async fn write_snapshot<T>(workspace_id: &str, snapshot: &T) -> Result<LinkSnapshotResponse>
where
    T: Serialize + ?Sized,
{
    let body = snapshot_body(snapshot)?;
    write_snapshot_bytes(workspace_id, body, None).await
}

/// Write a JSON snapshot to a workspace using an explicit Cloud API client.
#[tracing::instrument(skip(client, snapshot))]
pub async fn write_snapshot_with_client<T>(
    client: &reqwest::Client,
    workspace_id: &str,
    snapshot: &T,
) -> Result<LinkSnapshotResponse>
where
    T: Serialize + ?Sized,
{
    let body = snapshot_body(snapshot)?;
    write_snapshot_bytes_with_client(client, workspace_id, body, None).await
}

/// Write a raw JSON snapshot body to a workspace and create a link.
///
/// Use `encoding` when `body` is already gzip-compressed JSON. This uses the
/// configured Stencila API key as a bearer token.
#[tracing::instrument(skip(body))]
pub async fn write_snapshot_bytes(
    workspace_id: &str,
    body: impl Into<Vec<u8>>,
    encoding: Option<SnapshotContentEncoding>,
) -> Result<LinkSnapshotResponse> {
    let client = client().await?;
    write_snapshot_bytes_with_client(&client, workspace_id, body, encoding).await
}

/// Write a raw JSON snapshot body using an explicit Cloud API client.
#[tracing::instrument(skip(client, body))]
pub async fn write_snapshot_bytes_with_client(
    client: &reqwest::Client,
    workspace_id: &str,
    body: impl Into<Vec<u8>>,
    encoding: Option<SnapshotContentEncoding>,
) -> Result<LinkSnapshotResponse> {
    let body = body.into();
    validate_snapshot_size(body.len())?;

    let url = format!("{}/workspaces/{workspace_id}/links", base_url());

    tracing::debug!("Writing link snapshot to workspace {workspace_id}");

    let response = request_with_body(client.post(&url), body, encoding)
        .send()
        .await?;

    process_link_response(response).await
}

/// Write a pre-hashed JSON snapshot to a workspace.
///
/// The API verifies that `sha256` matches the canonicalized snapshot hash. This
/// uses the configured Stencila API key as a bearer token.
#[tracing::instrument(skip(snapshot))]
pub async fn write_prehashed_snapshot<T>(
    workspace_id: &str,
    sha256: &str,
    snapshot: &T,
) -> Result<LinkSnapshotResponse>
where
    T: Serialize + ?Sized,
{
    let client = client().await?;
    write_prehashed_snapshot_with_client(&client, workspace_id, sha256, snapshot).await
}

/// Write a pre-hashed JSON snapshot using an explicit Cloud API client.
#[tracing::instrument(skip(client, snapshot))]
pub async fn write_prehashed_snapshot_with_client<T>(
    client: &reqwest::Client,
    workspace_id: &str,
    sha256: &str,
    snapshot: &T,
) -> Result<LinkSnapshotResponse>
where
    T: Serialize + ?Sized,
{
    let body = snapshot_body(snapshot)?;
    write_prehashed_snapshot_bytes_with_client(client, workspace_id, sha256, body, None).await
}

/// Write a raw pre-hashed JSON snapshot body to a workspace.
///
/// Use `encoding` when `body` is already gzip-compressed JSON. This uses the
/// configured Stencila API key as a bearer token.
#[tracing::instrument(skip(body))]
pub async fn write_prehashed_snapshot_bytes(
    workspace_id: &str,
    sha256: &str,
    body: impl Into<Vec<u8>>,
    encoding: Option<SnapshotContentEncoding>,
) -> Result<LinkSnapshotResponse> {
    let client = client().await?;
    write_prehashed_snapshot_bytes_with_client(&client, workspace_id, sha256, body, encoding).await
}

/// Write a raw pre-hashed JSON snapshot body using an explicit Cloud API client.
#[tracing::instrument(skip(client, body))]
pub async fn write_prehashed_snapshot_bytes_with_client(
    client: &reqwest::Client,
    workspace_id: &str,
    sha256: &str,
    body: impl Into<Vec<u8>>,
    encoding: Option<SnapshotContentEncoding>,
) -> Result<LinkSnapshotResponse> {
    let body = body.into();
    validate_snapshot_size(body.len())?;

    let url = format!("{}/workspaces/{workspace_id}/objects/{sha256}", base_url());

    tracing::debug!("Writing pre-hashed link snapshot {sha256} to workspace {workspace_id}");

    let response = request_with_body(client.put(&url), body, encoding)
        .send()
        .await?;

    process_link_response(response).await
}

/// Check whether a snapshot object is already linked to a workspace.
///
/// Returns `Ok(Some(metadata))` for `204 No Content`, `Ok(None)` for `404 Not
/// Found`, and an error for other statuses. This uses the configured Stencila
/// API key as a bearer token.
#[tracing::instrument]
pub async fn check_snapshot(
    workspace_id: &str,
    sha256: &str,
) -> Result<Option<LinkSnapshotMetadata>> {
    let client = client().await?;
    check_snapshot_with_client(&client, workspace_id, sha256).await
}

/// Check whether a snapshot object is already linked using an explicit Cloud API client.
#[tracing::instrument(skip(client))]
pub async fn check_snapshot_with_client(
    client: &reqwest::Client,
    workspace_id: &str,
    sha256: &str,
) -> Result<Option<LinkSnapshotMetadata>> {
    let url = format!("{}/workspaces/{workspace_id}/objects/{sha256}", base_url());

    tracing::debug!("Checking link snapshot {sha256} in workspace {workspace_id}");

    let response = client.head(&url).send().await?;

    process_check_response(response).await
}

/// Read a JSON snapshot from a canonical link public ID.
///
/// This uses the configured Stencila API key as a bearer token.
#[tracing::instrument]
pub async fn read_snapshot(public_id: &str) -> Result<SnapshotEnvelope> {
    let client = client().await?;
    read_snapshot_with_client(&client, public_id).await
}

/// Read a JSON snapshot using an explicit Cloud API client.
#[tracing::instrument(skip(client))]
pub async fn read_snapshot_with_client(
    client: &reqwest::Client,
    public_id: &str,
) -> Result<SnapshotEnvelope> {
    let url = link_json_url(public_id);

    tracing::debug!("Reading link snapshot from {url}");

    let response = client.get(url).send().await?;

    process_response(response).await
}

fn snapshot_body<T>(snapshot: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let body = serde_json::to_vec(snapshot)?;
    validate_snapshot_size(body.len())?;
    Ok(body)
}

fn validate_snapshot_size(size: usize) -> Result<()> {
    if size > MAX_SNAPSHOT_SIZE {
        bail!(
            "Snapshot body is {size} bytes, which exceeds the {} byte limit",
            MAX_SNAPSHOT_SIZE
        );
    }

    Ok(())
}

fn request_with_body(
    request: reqwest::RequestBuilder,
    body: Vec<u8>,
    encoding: Option<SnapshotContentEncoding>,
) -> reqwest::RequestBuilder {
    let mut request = request
        .header(CONTENT_TYPE, "application/json")
        .header(CONTENT_LENGTH, body.len().to_string());

    if let Some(encoding) = encoding {
        request = request.header(CONTENT_ENCODING, encoding.as_str());
    }

    request.body(body)
}

async fn process_link_response(response: reqwest::Response) -> Result<LinkSnapshotResponse> {
    if !response.status().is_success() {
        return process_response(response).await;
    }

    let location = optional_header(response.headers(), &LOCATION)?;
    let mut link: LinkSnapshotResponse = process_response(response).await?;
    link.location = location;

    Ok(link)
}

async fn process_check_response(
    response: reqwest::Response,
) -> Result<Option<LinkSnapshotMetadata>> {
    match response.status() {
        StatusCode::NO_CONTENT => Ok(Some(LinkSnapshotMetadata {
            location: optional_header(response.headers(), &LOCATION)?,
        })),
        StatusCode::NOT_FOUND => Ok(None),
        _ => {
            check_response(response).await?;
            unreachable!("check_response should return an error for non-success responses")
        }
    }
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
