use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use eyre::{Result, bail, eyre};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;
use url::Url;

use stencila_codec::{
    Codec, EncodeOptions, PushDryRunFile, PushDryRunOptions, PushResult, stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_docx::DocxCodec;

/// Error type for Google Docs operations
#[derive(Debug, Error)]
pub enum GDocError {
    /// Google account not linked to Stencila account
    ///
    /// The user needs to connect their Google account via the Stencila Cloud picker.
    /// The `connect_url` may contain a tenant-specific URL for managed environments.
    #[error("Google account not linked. Use Google Picker to connect and grant access.")]
    NotLinked {
        /// URL to connect the Google account (if available, may be tenant-specific)
        connect_url: Option<String>,
    },

    /// Access denied to the document (403)
    ///
    /// The user needs to grant access via the Google Picker.
    /// The `doc_id` can be used to construct a picker URL.
    #[error("Access denied to Google Doc '{doc_id}'. Use Google Picker to grant access.")]
    AccessDenied {
        /// The Google Doc ID that access was denied to
        doc_id: String,
    },

    /// Document not found (404)
    ///
    /// The document doesn't exist or has been deleted. Unlike AccessDenied,
    /// this error should not trigger the picker flow.
    #[error(
        "Google Doc '{doc_id}' not found. The document may have been deleted or the URL is incorrect."
    )]
    NotFound {
        /// The Google Doc ID that was not found
        doc_id: String,
    },

    /// Other error
    #[error("{0}")]
    Other(#[from] eyre::Report),
}

/// Information about a Google Doc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GDocInfo {
    /// The Google Drive file ID
    pub id: String,
    /// The Google Docs URL
    pub url: Url,
}

/// Response from Google Drive API when creating or updating a file
#[derive(Deserialize)]
struct DriveFileResponse {
    id: String,
}

/// Response from Google Drive API when fetching file metadata
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveFileMetadata {
    modified_time: String,
}

/// Push a document to Google Docs
///
/// If `existing_url` is provided, updates the existing document.
/// Otherwise, creates a new document.
///
/// This function will obtain a Google Drive access token from Stencila Cloud.
///
/// Returns `GDocError::NotLinked` if the Google account is not connected.
/// Returns `GDocError::AccessDenied` if the app doesn't have access to the
/// document (when updating an existing document).
///
/// Returns a PushResult with the URL of the Google Doc on success.
pub async fn push(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    url: Option<&Url>,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    // Get access token only if not in dry-run mode (non-retrying so caller can handle errors)
    let access_token = if dry_run.is_none() {
        match stencila_cloud::google_get_token_once().await {
            Ok(token) => Some(token),
            Err(stencila_cloud::GoogleTokenError::NotLinked { connect_url }) => {
                return Err(GDocError::NotLinked { connect_url }.into());
            }
            Err(e) => {
                return Err(GDocError::Other(eyre::eyre!("{e}")).into());
            }
        }
    } else {
        None
    };

    if let Some(url) = url {
        // Update existing document
        let doc_id = extract_doc_id(url)?;
        update(node, path, access_token.as_deref(), &doc_id, dry_run).await
    } else {
        // Create new document
        upload(node, path, title, access_token.as_deref(), dry_run).await
    }
}

/// Upload a new document to Google Docs
async fn upload(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    access_token: Option<&str>,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    // Export document to DOCX in a temporary file
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    DocxCodec
        .to_path(
            node,
            temp_path,
            Some(EncodeOptions {
                format: Some(Format::GDocx),
                from_path: path.map(PathBuf::from),
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    // Handle dry-run mode
    if let Some(dry_run_opts) = dry_run {
        let metadata = tokio::fs::metadata(temp_path).await?;
        let file_size = metadata.len();

        let filename = format!("{}.docx", title.unwrap_or("Untitled"));

        let local_path = if let Some(output_dir) = &dry_run_opts.output_dir {
            let dest_path = output_dir.join(&filename);
            tokio::fs::create_dir_all(output_dir).await?;
            tokio::fs::copy(temp_path, &dest_path).await?;
            Some(dest_path)
        } else {
            None
        };

        let mock_url = Url::parse("https://docs.google.com/document/d/...")?;

        let dry_run_file = PushDryRunFile {
            storage_path: filename,
            local_path,
            size: file_size,
            compressed: false,
            route: None,
        };

        return Ok(PushResult::DryRun {
            url: mock_url,
            files: vec![dry_run_file],
            output_dir: dry_run_opts.output_dir,
        });
    }

    // Normal upload mode - access_token must be present
    let access_token = access_token.ok_or_else(|| eyre!("Access token required for upload"))?;

    // Read the DOCX file
    let docx_bytes = tokio::fs::read(temp_path).await?;

    // Create metadata part
    let metadata = serde_json::to_string(&serde_json::json!({
        "name": title,
        "mimeType": "application/vnd.google-apps.document",
    }))?;

    // Build multipart request
    let client = Client::new();
    let form = multipart::Form::new()
        .part(
            "metadata",
            multipart::Part::text(metadata).mime_str("application/json")?,
        )
        .part(
            "file",
            multipart::Part::bytes(docx_bytes).mime_str(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            )?,
        );

    // Send upload request
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .header("Authorization", format!("Bearer {access_token}"))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to upload to Google Docs ({status}): {error_text}");
    }

    // Parse response to get document ID
    let drive_response: DriveFileResponse = response.json().await?;

    // Construct Google Docs URL
    let url = Url::parse(&format!(
        "https://docs.google.com/document/d/{}",
        drive_response.id
    ))?;

    Ok(PushResult::Uploaded(url))
}

/// Update an existing Google Doc
async fn update(
    node: &Node,
    path: Option<&Path>,
    access_token: Option<&str>,
    doc_id: &str,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    // Export document to DOCX in a temporary file
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    DocxCodec
        .to_path(
            node,
            temp_path,
            Some(EncodeOptions {
                format: Some(Format::GDocx),
                from_path: path.map(PathBuf::from),
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    // Handle dry-run mode
    if let Some(dry_run_opts) = dry_run {
        let metadata = tokio::fs::metadata(temp_path).await?;
        let file_size = metadata.len();

        let filename = format!("{}.docx", doc_id);

        let local_path = if let Some(output_dir) = &dry_run_opts.output_dir {
            let dest_path = output_dir.join(&filename);
            tokio::fs::create_dir_all(output_dir).await?;
            tokio::fs::copy(temp_path, &dest_path).await?;
            Some(dest_path)
        } else {
            None
        };

        let url = Url::parse(&format!("https://docs.google.com/document/d/{doc_id}"))?;

        let dry_run_file = PushDryRunFile {
            storage_path: filename,
            local_path,
            size: file_size,
            compressed: false,
            route: None,
        };

        return Ok(PushResult::DryRun {
            url,
            files: vec![dry_run_file],
            output_dir: dry_run_opts.output_dir,
        });
    }

    // Normal update mode - access_token must be present
    let access_token = access_token.ok_or_else(|| eyre!("Access token required for update"))?;

    // Read the DOCX file
    let docx_bytes = tokio::fs::read(temp_path).await?;

    // Send update request
    let client = Client::new();
    let response = client
        .patch(format!(
            "https://www.googleapis.com/upload/drive/v3/files/{doc_id}?uploadType=media",
        ))
        .header("Authorization", format!("Bearer {access_token}"))
        .header(
            "Content-Type",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )
        .body(docx_bytes)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        // 404 or notFound -> document doesn't exist
        if status.as_u16() == 404 || error_text.contains("notFound") {
            return Err(GDocError::NotFound {
                doc_id: doc_id.to_string(),
            }
            .into());
        }

        // 403 or forbidden -> permission issue, picker can help
        if status.as_u16() == 403 || error_text.contains("forbidden") {
            return Err(GDocError::AccessDenied {
                doc_id: doc_id.to_string(),
            }
            .into());
        }

        bail!("Failed to update Google Doc ({status}): {error_text}");
    }

    // Construct Google Docs URL
    let url = Url::parse(&format!("https://docs.google.com/document/d/{doc_id}"))?;

    Ok(PushResult::Uploaded(url))
}

/// Pull a document from Google Docs
///
/// Downloads the document as DOCX and saves it to the specified path.
///
/// Returns `GDocError::NotLinked` if the Google account is not connected.
/// Returns `GDocError::AccessDenied` if the app doesn't have access to the
/// document. The caller should handle both by prompting the user via the
/// Google Picker, then retrying.
///
/// This function uses a non-retrying token fetch so callers can handle
/// connection errors appropriately (e.g., by opening the picker).
pub async fn pull(url: &Url, dest: &Path) -> Result<(), GDocError> {
    let doc_id = extract_doc_id(url)?;

    // Get access token from Stencila Cloud (non-retrying)
    let access_token = match stencila_cloud::google_get_token_once().await {
        Ok(token) => token,
        Err(stencila_cloud::GoogleTokenError::NotLinked { connect_url }) => {
            return Err(GDocError::NotLinked { connect_url });
        }
        Err(e) => {
            return Err(GDocError::Other(eyre::eyre!("{e}")));
        }
    };

    // Download the document as DOCX
    let client = Client::new();
    let response = client
        .get(format!(
            "https://www.googleapis.com/drive/v3/files/{doc_id}/export?mimeType=application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| GDocError::Other(e.into()))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();

        // 404 or notFound -> document doesn't exist
        if status.as_u16() == 404 || error_text.contains("notFound") {
            return Err(GDocError::NotFound { doc_id });
        }

        // 403 or forbidden -> permission issue, picker can help
        if status.as_u16() == 403 || error_text.contains("forbidden") {
            return Err(GDocError::AccessDenied { doc_id });
        }

        return Err(GDocError::Other(eyre::eyre!(
            "Failed to download from Google Docs ({status}): {error_text}"
        )));
    }

    // Write the downloaded bytes directly to the destination
    let bytes = response
        .bytes()
        .await
        .map_err(|error| GDocError::Other(error.into()))?;
    tokio::fs::write(dest, &bytes)
        .await
        .map_err(|error| GDocError::Other(error.into()))?;

    // Pre-process the DOCX to restore inline code styling that may have been
    // lost in Google Docs (which doesn't support character styles like "Verbatim Char")
    stencila_codec_docx::preprocess::restore_verbatim_char_style(dest).map_err(GDocError::Other)?;

    Ok(())
}

/// Time that a Google Doc was last modified as a Unix timestamp
///
/// This function will obtain a Google Drive access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
pub async fn modified_at(url: &Url) -> Result<u64> {
    let access_token = stencila_cloud::get_token("google").await?;
    let doc_id = extract_doc_id(url)?;

    // Fetch file metadata with only the modifiedTime field
    let client = Client::new();
    let response = client
        .get(format!(
            "https://www.googleapis.com/drive/v3/files/{doc_id}?fields=modifiedTime"
        ))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to fetch Google Doc metadata ({status}): {error_text}");
    }

    // Parse response to get modified time
    let metadata: DriveFileMetadata = response.json().await?;

    // Parse ISO 8601 timestamp and convert to Unix timestamp
    let modified_time = DateTime::parse_from_rfc3339(&metadata.modified_time)?
        .with_timezone(&Utc)
        .timestamp() as u64;

    Ok(modified_time)
}

/// Extract the document ID from a Google Docs URL
///
/// Supports URLs in the format:
/// - https://docs.google.com/document/d/{id}/edit
/// - https://docs.google.com/document/d/{id}
pub fn extract_doc_id(url: &Url) -> Result<String> {
    // Check that it's a Google Docs URL
    if url.host_str() != Some("docs.google.com") {
        bail!("Not a Google Docs URL: {url}");
    }

    // Parse path segments
    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| eyre!("Invalid URL"))?
        .collect();

    // Expected format: /document/d/{id}/... or /document/d/{id}
    if segments.len() >= 3 && segments[0] == "document" && segments[1] == "d" {
        Ok(segments[2].to_string())
    } else {
        bail!("Invalid Google Docs URL format: {url}");
    }
}
