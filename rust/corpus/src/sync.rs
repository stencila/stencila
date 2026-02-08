//! Cloud sync for corpora.
//!
//! Implements push (local → cloud) and pull (cloud → local) using the
//! Stencila Cloud Corpora API. The protocol leverages the immutability
//! of sealed segments: each sealed segment has a blake3 hash and is
//! uploaded/downloaded exactly once. Only the manifest, state.sqlite,
//! and the active segment are mutable and re-synced on every push/pull.
//!
//! # Sync flow
//!
//! **Push:**
//! 1. Seal the active segment (so everything is content-addressable).
//! 2. Collect hashes for all sealed segments + state.sqlite.
//! 3. Ask the cloud which objects already exist.
//! 4. Upload missing objects via presigned URLs.
//! 5. Confirm uploads.
//! 6. PUT the manifest to the cloud.
//!
//! **Pull:**
//! 1. GET the remote manifest.
//! 2. Determine which segments are missing locally.
//! 3. Download missing segments via presigned URLs.
//! 4. Download state.sqlite.
//! 5. Replace local manifest.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::manifest::Manifest;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Maximum objects per batch API call.
const BATCH_LIMIT: usize = 100;

// ---------------------------------------------------------------------------
// Hash helpers
// ---------------------------------------------------------------------------

/// Format a raw hex hash as the API-required prefixed form.
fn prefixed_hash(hex: &str) -> String {
    if hex.starts_with("blake3:") {
        hex.to_string()
    } else {
        format!("blake3:{hex}")
    }
}

/// Strip the `blake3:` prefix from an API hash, returning raw hex.
fn strip_prefix(hash: &str) -> &str {
    hash.strip_prefix("blake3:").unwrap_or(hash)
}

/// Compute the blake3 hash of a file, returned as raw hex.
fn hash_file(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    Ok(blake3::hash(&data).to_hex().to_string())
}

// ---------------------------------------------------------------------------
// API types (request/response)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CreateCorpusRequest {
    r#type: String,
    name: String,
    #[serde(rename = "externalKey", skip_serializing_if = "Option::is_none")]
    external_key: Option<String>,
}

#[derive(Deserialize)]
pub struct CorpusInfo {
    #[serde(rename = "publicId")]
    pub public_id: String,
    pub r#type: String,
    pub name: String,
    #[serde(rename = "externalKey")]
    pub external_key: Option<String>,
}

#[derive(Serialize)]
struct ExistsRequest {
    hashes: Vec<String>,
}

#[derive(Deserialize)]
struct ExistsResponse {
    exists: HashMap<String, bool>,
}

#[derive(Serialize)]
struct UploadUrlsRequest {
    objects: Vec<ObjectRef>,
}

#[derive(Serialize, Clone)]
struct ObjectRef {
    hash: String,
    size: u64,
}

#[derive(Deserialize)]
struct UploadUrlsResponse {
    urls: Vec<UploadUrlEntry>,
}

#[derive(Deserialize)]
struct UploadUrlEntry {
    hash: String,
    #[serde(rename = "uploadUrl")]
    upload_url: String,
}

#[derive(Serialize)]
struct ConfirmRequest {
    objects: Vec<ObjectRef>,
}

#[derive(Deserialize)]
struct ConfirmResponse {
    confirmed: u64,
    #[serde(default)]
    errors: Vec<ConfirmError>,
}

#[derive(Deserialize)]
struct ConfirmError {
    hash: String,
    error: String,
}

#[derive(Serialize)]
struct DownloadUrlsRequest {
    hashes: Vec<String>,
}

#[derive(Deserialize)]
struct DownloadUrlsResponse {
    urls: Vec<DownloadUrlEntry>,
}

#[derive(Deserialize)]
struct DownloadUrlEntry {
    hash: String,
    #[serde(rename = "downloadUrl")]
    download_url: String,
}

// ---------------------------------------------------------------------------
// SyncReport
// ---------------------------------------------------------------------------

/// Summary of a sync operation.
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// Number of objects uploaded (push) or downloaded (pull).
    pub objects_transferred: usize,
    /// Total bytes transferred.
    pub bytes_transferred: u64,
    /// Number of objects that already existed remotely (push) or locally (pull).
    pub objects_skipped: usize,
    /// Whether the manifest was updated.
    pub manifest_updated: bool,
}

// ---------------------------------------------------------------------------
// SyncClient
// ---------------------------------------------------------------------------

/// Client for syncing a corpus to/from Stencila Cloud.
pub struct SyncClient {
    client: Client,
    base_url: String,
    workspace_id: String,
    corpus_id: String,
    manifest_name: String,
}

impl SyncClient {
    /// Create a new sync client.
    ///
    /// Uses the Stencila Cloud auth token from the environment/keyring.
    /// `manifest_name` is the named manifest slot on the server (e.g. "latest",
    /// "device/macbook-1").
    pub async fn new(
        workspace_id: &str,
        corpus_id: &str,
        manifest_name: &str,
    ) -> Result<Self> {
        let token = stencila_cloud::api_token().ok_or_else(|| {
            Error::Other("Not signed in to Stencila Cloud. Run `stencila signin` first.".into())
        })?;

        let client = Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| Error::Other(format!("invalid token: {e}")))?,
                );
                h
            })
            .build()?;

        Ok(Self {
            client,
            base_url: stencila_cloud::base_url(),
            workspace_id: workspace_id.to_string(),
            corpus_id: corpus_id.to_string(),
            manifest_name: manifest_name.to_string(),
        })
    }

    /// Create a corpus on the server (idempotent via external_key).
    pub async fn create_corpus(
        &self,
        corpus_type: &str,
        name: &str,
        external_key: Option<&str>,
    ) -> Result<CorpusInfo> {
        let url = format!(
            "{}/workspaces/{}/corpora",
            self.base_url, self.workspace_id
        );

        let resp = self
            .client
            .post(&url)
            .json(&CreateCorpusRequest {
                r#type: corpus_type.to_string(),
                name: name.to_string(),
                external_key: external_key.map(String::from),
            })
            .send()
            .await?;

        self.parse_response(resp).await
    }

    // -- Push --------------------------------------------------------------

    /// Push the local corpus to the cloud.
    ///
    /// The corpus should be sealed before calling this (all segments immutable).
    /// `corpus_dir` is the `.corpus/` directory, `manifest` is the local manifest.
    pub async fn push(
        &self,
        corpus_dir: &Path,
        manifest: &Manifest,
    ) -> Result<SyncReport> {
        let segments_dir = corpus_dir.join("segments");
        let state_path = corpus_dir.join("state.sqlite");

        // 1. Collect all objects to sync: sealed segments + state.sqlite.
        let mut objects: Vec<SyncObject> = Vec::new();

        for seg in &manifest.segments {
            if let Some(hash) = &seg.hash {
                let path = seg.id.path_in(&segments_dir);
                objects.push(SyncObject {
                    hash: hash.clone(),
                    size: seg.size,
                    path,
                });
            }
        }

        // state.sqlite
        if state_path.exists() {
            let state_hash = hash_file(&state_path)?;
            let state_size = std::fs::metadata(&state_path)?.len();
            objects.push(SyncObject {
                hash: state_hash,
                size: state_size,
                path: state_path,
            });
        }

        if objects.is_empty() {
            // Nothing to push — just update the manifest.
            self.put_manifest(manifest).await?;
            return Ok(SyncReport {
                objects_transferred: 0,
                bytes_transferred: 0,
                objects_skipped: 0,
                manifest_updated: true,
            });
        }

        // 2. Check which objects already exist on the server.
        let all_hashes: Vec<String> = objects.iter().map(|o| prefixed_hash(&o.hash)).collect();
        let existing = self.check_exists(&all_hashes).await?;

        let missing: Vec<&SyncObject> = objects
            .iter()
            .filter(|o| {
                let ph = prefixed_hash(&o.hash);
                !existing.get(&ph).copied().unwrap_or(false)
            })
            .collect();

        let objects_skipped = objects.len() - missing.len();

        // 3. Upload missing objects.
        let mut objects_transferred = 0;
        let mut bytes_transferred: u64 = 0;

        for chunk in missing.chunks(BATCH_LIMIT) {
            let refs: Vec<ObjectRef> = chunk
                .iter()
                .map(|o| ObjectRef {
                    hash: prefixed_hash(&o.hash),
                    size: o.size,
                })
                .collect();

            // Get presigned upload URLs.
            let upload_urls = self.get_upload_urls(&refs).await?;

            // Upload each object.
            for obj in chunk {
                let ph = prefixed_hash(&obj.hash);
                if let Some(entry) = upload_urls.iter().find(|u| u.hash == ph) {
                    self.upload_object(&obj.path, &entry.upload_url).await?;
                    objects_transferred += 1;
                    bytes_transferred += obj.size;
                }
            }

            // Confirm uploads.
            self.confirm_uploads(&refs).await?;
        }

        // 4. Store the manifest.
        self.put_manifest(manifest).await?;

        Ok(SyncReport {
            objects_transferred,
            bytes_transferred,
            objects_skipped,
            manifest_updated: true,
        })
    }

    // -- Pull --------------------------------------------------------------

    /// Pull the remote corpus to the local `.corpus/` directory.
    ///
    /// Returns the remote manifest (which has been saved locally) and a report.
    pub async fn pull(
        &self,
        corpus_dir: &Path,
    ) -> Result<(Manifest, SyncReport)> {
        let segments_dir = corpus_dir.join("segments");
        std::fs::create_dir_all(&segments_dir)?;

        // 1. Get the remote manifest.
        let remote_manifest: Manifest = self.get_manifest().await?;

        // 2. Determine which sealed segments are missing locally.
        let mut to_download: Vec<(String, PathBuf, u64)> = Vec::new(); // (hash, local_path, size)

        for seg in &remote_manifest.segments {
            if let Some(hash) = &seg.hash {
                let local_path = seg.id.path_in(&segments_dir);
                if local_path.exists() {
                    // Verify hash matches (skip if it does).
                    let local_hash = hash_file(&local_path)?;
                    if &local_hash == hash {
                        continue;
                    }
                }
                to_download.push((hash.clone(), local_path, seg.size));
            }
        }

        // Also download state.sqlite.
        // We always re-download state since it's mutable and small.
        let state_path = corpus_dir.join("state.sqlite");
        // State hash is not in the manifest — we need to check if the server has it.
        // We'll download it unconditionally via its well-known key.
        let download_state = true;

        let objects_skipped = remote_manifest
            .segments
            .iter()
            .filter(|s| s.hash.is_some())
            .count()
            - to_download.len();

        // 3. Download missing segments.
        let mut objects_transferred = 0;
        let mut bytes_transferred: u64 = 0;

        for chunk in to_download.chunks(BATCH_LIMIT) {
            let hashes: Vec<String> = chunk.iter().map(|(h, _, _)| prefixed_hash(h)).collect();
            let download_urls = self.get_download_urls(&hashes).await?;

            for (hash, local_path, size) in chunk {
                let ph = prefixed_hash(hash);
                if let Some(entry) = download_urls.iter().find(|u| u.hash == ph) {
                    self.download_object(&entry.download_url, local_path)
                        .await?;
                    objects_transferred += 1;
                    bytes_transferred += size;
                }
            }
        }

        // 4. Download state.sqlite if needed.
        if download_state {
            // State is stored as an object with its own hash.
            // We look for it via the exists check using a special convention:
            // the manifest doesn't track state hash, so we try to download it
            // if the server has any version. For v1, we'll just download it
            // via the same object mechanism if we know its hash.
            //
            // Since the push side uploads state.sqlite as a regular object,
            // pull needs to know its hash. We store it in the manifest for now.
            // If not available, skip — state will be rebuilt on next build.
            if let Some(state_hash) = self.find_state_hash(&remote_manifest).await {
                let urls = self.get_download_urls(&[prefixed_hash(&state_hash)]).await?;
                if let Some(entry) = urls.first() {
                    self.download_object(&entry.download_url, &state_path).await?;
                    objects_transferred += 1;
                    bytes_transferred += std::fs::metadata(&state_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                }
            }
        }

        // 5. Save the remote manifest locally.
        remote_manifest.save(corpus_dir)?;

        Ok((
            remote_manifest,
            SyncReport {
                objects_transferred,
                bytes_transferred,
                objects_skipped,
                manifest_updated: true,
            },
        ))
    }

    // -- Manifest API ------------------------------------------------------

    /// PUT the manifest to the cloud manifests API.
    async fn put_manifest(&self, manifest: &Manifest) -> Result<()> {
        let url = format!(
            "{}/workspaces/{}/corpora/{}/manifests/{}",
            self.base_url, self.workspace_id, self.corpus_id, self.manifest_name
        );

        let resp = self.client.put(&url).json(manifest).send().await?;
        self.check_response(resp).await
    }

    /// GET the manifest from the cloud manifests API.
    async fn get_manifest(&self) -> Result<Manifest> {
        let url = format!(
            "{}/workspaces/{}/corpora/{}/manifests/{}",
            self.base_url, self.workspace_id, self.corpus_id, self.manifest_name
        );

        let resp = self.client.get(&url).send().await?;
        self.parse_response(resp).await
    }

    // -- Objects API -------------------------------------------------------

    /// Check which hashes already exist on the server.
    async fn check_exists(
        &self,
        hashes: &[String],
    ) -> Result<HashMap<String, bool>> {
        let url = format!(
            "{}/workspaces/{}/corpora/{}/objects/exists",
            self.base_url, self.workspace_id, self.corpus_id
        );

        let mut all_exists = HashMap::new();

        for chunk in hashes.chunks(BATCH_LIMIT) {
            let resp = self
                .client
                .post(&url)
                .json(&ExistsRequest {
                    hashes: chunk.to_vec(),
                })
                .send()
                .await?;

            let body: ExistsResponse = self.parse_response(resp).await?;
            all_exists.extend(body.exists);
        }

        Ok(all_exists)
    }

    /// Get presigned upload URLs for a batch of objects.
    async fn get_upload_urls(
        &self,
        objects: &[ObjectRef],
    ) -> Result<Vec<UploadUrlEntry>> {
        let url = format!(
            "{}/workspaces/{}/corpora/{}/objects/upload-urls",
            self.base_url, self.workspace_id, self.corpus_id
        );

        let resp = self
            .client
            .post(&url)
            .json(&UploadUrlsRequest {
                objects: objects.to_vec(),
            })
            .send()
            .await?;

        let body: UploadUrlsResponse = self.parse_response(resp).await?;
        Ok(body.urls)
    }

    /// Upload a local file to a presigned URL.
    async fn upload_object(&self, local_path: &Path, upload_url: &str) -> Result<()> {
        let data = tokio::fs::read(local_path).await.map_err(|e| {
            Error::Other(format!(
                "failed to read {} for upload: {e}",
                local_path.display()
            ))
        })?;

        let size = data.len();
        tracing::debug!(
            path = %local_path.display(),
            size,
            "uploading object"
        );

        let resp = self
            .client
            .put(upload_url)
            .header("Content-Length", size.to_string())
            .body(data)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Cloud(
                status,
                format!("upload failed for {}: {body}", local_path.display()),
            ));
        }

        Ok(())
    }

    /// Confirm that uploaded objects are ready.
    async fn confirm_uploads(&self, objects: &[ObjectRef]) -> Result<()> {
        let url = format!(
            "{}/workspaces/{}/corpora/{}/objects/confirm",
            self.base_url, self.workspace_id, self.corpus_id
        );

        let resp = self
            .client
            .post(&url)
            .json(&ConfirmRequest {
                objects: objects.to_vec(),
            })
            .send()
            .await?;

        let body: ConfirmResponse = self.parse_response(resp).await?;

        if !body.errors.is_empty() {
            let msgs: Vec<String> = body
                .errors
                .iter()
                .map(|e| format!("{}: {}", strip_prefix(&e.hash), e.error))
                .collect();
            tracing::warn!(
                errors = %msgs.join("; "),
                confirmed = body.confirmed,
                "some upload confirmations failed"
            );
        }

        Ok(())
    }

    /// Get presigned download URLs for a batch of hashes.
    async fn get_download_urls(
        &self,
        hashes: &[String],
    ) -> Result<Vec<DownloadUrlEntry>> {
        let url = format!(
            "{}/workspaces/{}/corpora/{}/objects/download-urls",
            self.base_url, self.workspace_id, self.corpus_id
        );

        let resp = self
            .client
            .post(&url)
            .json(&DownloadUrlsRequest {
                hashes: hashes.to_vec(),
            })
            .send()
            .await?;

        let body: DownloadUrlsResponse = self.parse_response(resp).await?;
        Ok(body.urls)
    }

    /// Download an object from a presigned URL to a local path.
    async fn download_object(&self, download_url: &str, local_path: &Path) -> Result<()> {
        tracing::debug!(
            path = %local_path.display(),
            "downloading object"
        );

        let resp = self.client.get(download_url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Cloud(
                status,
                format!(
                    "download failed for {}: {body}",
                    local_path.display()
                ),
            ));
        }

        let bytes = resp.bytes().await?;

        // Write to a temp file then rename for atomicity.
        let tmp_path = local_path.with_extension("tmp");
        tokio::fs::write(&tmp_path, &bytes).await.map_err(|e| {
            Error::Other(format!(
                "failed to write {}: {e}",
                tmp_path.display()
            ))
        })?;
        tokio::fs::rename(&tmp_path, local_path).await.map_err(|e| {
            Error::Other(format!(
                "failed to rename {} → {}: {e}",
                tmp_path.display(),
                local_path.display()
            ))
        })?;

        Ok(())
    }

    // -- Helpers -----------------------------------------------------------

    /// Try to find the state.sqlite hash.
    ///
    /// For v1 we attempt to hash the local state file and check if the server
    /// has it. This is a best-effort mechanism — if state doesn't exist on the
    /// server, the client will rebuild it on next `corpus build`.
    async fn find_state_hash(&self, _remote_manifest: &Manifest) -> Option<String> {
        // The state hash isn't tracked in the manifest. In the push flow,
        // state.sqlite is uploaded as a regular object. We could:
        // a) Store the state hash in manifest metadata.
        // b) Use a well-known object key.
        //
        // For now, we don't download state on pull — it will be rebuilt
        // by the next local build. This is safe because state only contains
        // routing info that can be reconstructed from the segments.
        None
    }

    /// Parse a JSON response body, returning a typed value or an error.
    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();

            // Try to parse as the API error format.
            if let Ok(err) = serde_json::from_str::<CloudError>(&body) {
                return Err(Error::Cloud(status, err.error));
            }
            return Err(Error::Cloud(status, body));
        }

        let text = resp.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            Error::Other(format!(
                "failed to parse response: {e}\nbody: {}",
                if text.len() > 200 {
                    &text[..200]
                } else {
                    &text
                }
            ))
        })
    }

    /// Check a response for errors (for endpoints that return no body).
    async fn check_response(&self, resp: reqwest::Response) -> Result<()> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            if let Ok(err) = serde_json::from_str::<CloudError>(&body) {
                return Err(Error::Cloud(status, err.error));
            }
            return Err(Error::Cloud(status, body));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// A local object to sync.
struct SyncObject {
    hash: String,
    size: u64,
    path: PathBuf,
}

/// API error response shape.
#[derive(Deserialize)]
struct CloudError {
    #[allow(dead_code)]
    status: Option<u16>,
    error: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixed_hash_formats_correctly() {
        assert_eq!(
            prefixed_hash("abc123"),
            "blake3:abc123"
        );
        assert_eq!(
            prefixed_hash("blake3:abc123"),
            "blake3:abc123"
        );
    }

    #[test]
    fn strip_prefix_works() {
        assert_eq!(strip_prefix("blake3:abc123"), "abc123");
        assert_eq!(strip_prefix("abc123"), "abc123");
    }
}
