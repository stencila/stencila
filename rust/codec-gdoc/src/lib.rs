use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use eyre::{Result, bail, eyre};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use url::Url;

use stencila_codec::{Codec, EncodeOptions, stencila_format::Format, stencila_schema::Node};
use stencila_codec_docx::DocxCodec;

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
/// This function will obtain a Google Drive access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
///
/// Returns the URL of the Google Doc.
pub async fn push(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    url: Option<&Url>,
) -> Result<Url> {
    let access_token = stencila_cloud::get_token("google").await?;

    if let Some(url) = url {
        // Update existing document
        let doc_id = extract_doc_id(url)?;
        update(node, path, &access_token, &doc_id).await
    } else {
        // Create new document
        upload(node, path, title, &access_token).await
    }
}

/// Upload a new document to Google Docs
async fn upload(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    access_token: &str,
) -> Result<Url> {
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

    Ok(url)
}

/// Update an existing Google Doc
async fn update(node: &Node, path: Option<&Path>, access_token: &str, doc_id: &str) -> Result<Url> {
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
        bail!("Failed to update Google Doc ({status}): {error_text}");
    }

    // Construct Google Docs URL
    let url = Url::parse(&format!("https://docs.google.com/document/d/{doc_id}"))?;

    Ok(url)
}

/// Pull a document from Google Docs
///
/// Downloads the document as DOCX and saves it to the specified path.
///
/// This function will obtain a Google Drive access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
pub async fn pull(url: &Url, dest: &Path) -> Result<()> {
    // Get access token from Stencila Cloud
    let access_token = stencila_cloud::get_token("google").await?;

    // Extract document ID
    let doc_id = extract_doc_id(url)?;

    // Download the document as DOCX
    let client = Client::new();
    let response = client
        .get(format!(
            "https://www.googleapis.com/drive/v3/files/{doc_id}/export?mimeType=application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to download from Google Docs ({status}): {error_text}");
    }

    // Write the downloaded bytes directly to the destination
    let bytes = response.bytes().await?;
    tokio::fs::write(dest, bytes).await?;

    Ok(())
}

/// Get metadata for a Google Doc
///
/// Returns the last modified time as a Unix timestamp.
///
/// This function will obtain a Google Drive access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
pub async fn get_metadata(url: &Url) -> Result<u64> {
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
