//! Codec for Microsoft 365 / OneDrive integration
//!
//! # Current Implementation
//!
//! This codec uploads files to the **OneDrive App Folder**, a special folder that is:
//! - Only accessible to this application
//! - Located at `Apps/Stencila/` in the user's OneDrive
//! - Requires only the `Files.ReadWrite.AppFolder` scope
//!
//! # Upgrading to Full OneDrive Access
//!
//! To allow users to upload to their main OneDrive folder instead of the app folder:
//! 1. Update the Stencila Cloud backend OAuth flow to request `Files.ReadWrite` scope
//! 2. Change the upload endpoint from `/special/approot` to `/items/root`
//! 3. Users will need to re-authorize the Microsoft connection
//!
//! The app folder approach works but is less discoverable for users since files are in
//! a hidden application-specific folder rather than their main OneDrive directory.

use std::path::Path;

use chrono::{DateTime, Utc};
use eyre::{Result, bail};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::Client;
use serde::Deserialize;
use tempfile::NamedTempFile;
use url::Url;

/// URL encoding set for OneDrive file paths
/// Encodes control characters and special characters but preserves common filename characters
const ONEDRIVE_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'/')
    .add(b'?')
    .add(b'[')
    .add(b']');

use stencila_codec::{Codec, EncodeOptions, stencila_schema::Node};
use stencila_codec_docx::DocxCodec;

/// Response from Microsoft Graph API when uploading a file
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveItemResponse {
    #[allow(dead_code)]
    id: String,
    web_url: String,
}

/// Response from Microsoft Graph API when fetching item metadata
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveItemMetadata {
    last_modified_date_time: String,
}

/// Push a document to Microsoft 365 / OneDrive
///
/// If `url` is provided, updates the existing document.
/// Otherwise, creates a new document.
///
/// This function will obtain a Microsoft access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
///
/// Returns the URL of the OneDrive document.
pub async fn push(node: &Node, title: Option<&str>, url: Option<&Url>) -> Result<Url> {
    let access_token = stencila_cloud::get_token("microsoft").await?;

    if let Some(url) = url {
        // Update existing document by extracting filename from URL
        let filename = extract_filename(url)?;
        // Use upload which will overwrite the existing file
        upload(node, &filename, &access_token).await
    } else {
        // Create new document
        let title = title.unwrap_or("Untitled.docx");
        // Ensure .docx extension
        let filename = if title.ends_with(".docx") {
            title.to_string()
        } else {
            format!("{title}.docx")
        };
        upload(node, &filename, &access_token).await
    }
}

/// Upload a document to OneDrive app folder
///
/// This function creates a new file or overwrites an existing file with the same name.
/// Using PUT with the same path allows both create and update operations.
async fn upload(node: &Node, filename: &str, access_token: &str) -> Result<Url> {
    // Export document to DOCX in a temporary file
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    DocxCodec
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

    // URL encode the filename for OneDrive path
    let encoded_filename = utf8_percent_encode(filename, ONEDRIVE_ENCODE_SET).to_string();

    // Upload to OneDrive app folder (special/approot)
    // This requires only Files.ReadWrite.AppFolder scope
    // To use the main OneDrive folder, the backend needs Files.ReadWrite scope
    let client = Client::new();
    let response = client
        .put(format!(
            "https://graph.microsoft.com/v1.0/me/drive/special/approot:/{encoded_filename}:/content"
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

        // Provide more helpful error messages based on status code
        match status.as_u16() {
            404 => bail!(
                "OneDrive app folder not found ({status}). This may indicate:\n  \
                - Your Microsoft account doesn't have OneDrive enabled\n  \
                - The app folder hasn't been initialized yet\n\n\
                Note: Files are uploaded to a Stencila app folder (requires Files.ReadWrite.AppFolder scope).\n\
                Error details: {error_text}"
            ),
            401 | 403 => bail!(
                "Access denied ({status}). Please ensure your Microsoft account is properly connected \
                and has granted OneDrive permissions.\n\n\
                Error details: {error_text}"
            ),
            423 => bail!(
                "File is locked ({status}). The document is currently being edited in OneDrive/Office.\n\n\
                Please close the document and try again.\n\n\
                Error details: {error_text}"
            ),
            _ => bail!("Failed to upload to OneDrive ({status}): {error_text}"),
        }
    }

    // Parse response to get document URL
    let drive_response: DriveItemResponse = response.json().await?;
    let url = Url::parse(&drive_response.web_url)?;

    Ok(url)
}

/// Pull a document from Microsoft 365 / OneDrive
///
/// Downloads the document as DOCX and saves it to the specified path.
///
/// This function will obtain a Microsoft access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
pub async fn pull(url: &Url, dest: &Path) -> Result<()> {
    // Get access token from Stencila Cloud
    let access_token = stencila_cloud::get_token("microsoft").await?;

    // Extract item ID
    let item_id = extract_item_id(url)?;

    // Download the document
    let client = Client::new();
    let response = client
        .get(format!(
            "https://graph.microsoft.com/v1.0/me/drive/items/{item_id}/content"
        ))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            404 => bail!(
                "OneDrive document not found ({status}). Please check the URL is correct \
                and the document hasn't been deleted.\n\n\
                Error details: {error_text}"
            ),
            401 | 403 => bail!(
                "Access denied ({status}). You may not have permission to access this document.\n\n\
                Error details: {error_text}"
            ),
            423 => bail!(
                "File is locked ({status}). The document is currently being edited in OneDrive/Office.\n\n\
                Please close the document and try again.\n\n\
                Error details: {error_text}"
            ),
            _ => bail!("Failed to download from OneDrive ({status}): {error_text}"),
        }
    }

    // Write the downloaded bytes directly to the destination
    let bytes = response.bytes().await?;
    tokio::fs::write(dest, bytes).await?;

    Ok(())
}

/// Get metadata for a Microsoft 365 / OneDrive document
///
/// Returns the last modified time as a Unix timestamp.
///
/// This function will obtain a Microsoft access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
pub async fn get_metadata(url: &Url) -> Result<u64> {
    let access_token = stencila_cloud::get_token("microsoft").await?;
    let item_id = extract_item_id(url)?;

    // Fetch item metadata with only the lastModifiedDateTime field
    let client = Client::new();
    let response = client
        .get(format!(
            "https://graph.microsoft.com/v1.0/me/drive/items/{item_id}?select=lastModifiedDateTime"
        ))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            404 => bail!(
                "OneDrive document not found ({status}). Please check the URL is correct \
                and the document hasn't been deleted.\n\n\
                Error details: {error_text}"
            ),
            401 | 403 => bail!(
                "Access denied ({status}). You may not have permission to access this document.\n\n\
                Error details: {error_text}"
            ),
            _ => bail!("Failed to fetch OneDrive metadata ({status}): {error_text}"),
        }
    }

    // Parse response to get modified time
    let metadata: DriveItemMetadata = response.json().await?;

    // Parse ISO 8601 timestamp and convert to Unix timestamp
    let modified_time = DateTime::parse_from_rfc3339(&metadata.last_modified_date_time)?
        .with_timezone(&Utc)
        .timestamp() as u64;

    Ok(modified_time)
}

/// Extract the filename from a OneDrive or SharePoint URL
///
/// Looks for the `file` query parameter in URLs like:
/// - https://{tenant}-my.sharepoint.com/...?file=document.docx&...
fn extract_filename(url: &Url) -> Result<String> {
    for (key, value) in url.query_pairs() {
        if key == "file" {
            return Ok(value.to_string());
        }
    }

    bail!(
        "Could not extract filename from URL: {url}. \
        The URL should contain a 'file' parameter with the document name."
    );
}

/// Extract the item ID from a OneDrive or SharePoint URL
///
/// Supports various URL formats:
/// - https://onedrive.live.com/?id={id}
/// - https://{tenant}-my.sharepoint.com/...?id={id}
/// - https://{tenant}-my.sharepoint.com/...?sourcedoc={guid} (SharePoint web URLs)
pub fn extract_item_id(url: &Url) -> Result<String> {
    // Try to extract from query parameters
    for (key, value) in url.query_pairs() {
        if key == "id" {
            return Ok(value.to_string());
        }

        // SharePoint web URLs use 'sourcedoc' parameter with a GUID
        // Format: sourcedoc=%7B<GUID>%7D or sourcedoc={<GUID>}
        if key == "sourcedoc" {
            // Remove URL encoding and curly braces if present
            let decoded = value.to_string();
            let cleaned = decoded
                .trim_start_matches('{')
                .trim_start_matches("%7B")
                .trim_end_matches('}')
                .trim_end_matches("%7D")
                .to_string();
            return Ok(cleaned);
        }
    }

    // If no recognized query parameter, try to extract from path for other formats
    bail!(
        "Could not extract item ID from URL: {}. \
        Supported URL formats include OneDrive sharing links with 'id' or 'sourcedoc' parameters.",
        url
    );
}
