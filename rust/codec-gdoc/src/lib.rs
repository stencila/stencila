use std::path::Path;

use eyre::{Result, bail, eyre};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use url::Url;

use stencila_codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    async_trait, stencila_format::Format, stencila_schema::Node,
};
use stencila_codec_docx::DocxCodec;

/// A codec for uploads/downloads to/from Google Docs
pub struct GDocCodec;

#[async_trait]
impl Codec for GDocCodec {
    fn name(&self) -> &str {
        "gdoc"
    }

    fn availability(&self) -> CodecAvailability {
        DocxCodec.availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::GDocx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::GDocx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let options = DecodeOptions {
            format: Some(Format::GDocx),
            ..options.unwrap_or_default()
        };

        DocxCodec.from_path(path, Some(options)).await
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = EncodeOptions {
            format: Some(Format::GDocx),
            ..options.unwrap_or_default()
        };

        DocxCodec.to_path(node, path, Some(options)).await
    }
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

/// Push a document to Google Docs
///
/// If `existing_url` is provided, updates the existing document.
/// Otherwise, creates a new document.
///
/// This function will obtain a Google Drive access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
///
/// Returns the URL of the Google Doc.
pub async fn push(node: &Node, title: Option<&str>, url: Option<&Url>) -> Result<Url> {
    let access_token = stencila_cloud::get_token("google").await?;

    if let Some(url) = url {
        // Update existing document
        let doc_id = extract_doc_id(url)?;
        update(node, &access_token, &doc_id).await
    } else {
        // Create new document
        let title = title.unwrap_or("Untitled");
        upload(node, title, &access_token).await
    }
}

/// Upload a new document to Google Docs
async fn upload(node: &Node, title: &str, access_token: &str) -> Result<Url> {
    // Export document to DOCX in a temporary file
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    GDocCodec
        .to_path(
            node,
            temp_path,
            Some(EncodeOptions {
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
async fn update(node: &Node, access_token: &str, doc_id: &str) -> Result<Url> {
    // Export document to DOCX in a temporary file
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    GDocCodec
        .to_path(
            node,
            temp_path,
            Some(EncodeOptions {
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

/// Extract the document ID from a Google Docs URL
///
/// Supports URLs in the format:
/// - https://docs.google.com/document/d/{id}/edit
/// - https://docs.google.com/document/d/{id}
pub fn extract_doc_id(url: &Url) -> Result<String> {
    // Check that it's a Google Docs URL
    if url.host_str() != Some("docs.google.com") {
        bail!("Not a Google Docs URL: {}", url);
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
        bail!("Invalid Google Docs URL format: {}", url);
    }
}
