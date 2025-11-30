//! Email attachments remote provider for Stencila
//!
//! This module provides pull functionality for retrieving DOCX attachments
//! from the Stencila Cloud email-to-PR feature.

use std::path::{Path, PathBuf};

use eyre::{Result, bail, eyre};
use indexmap::IndexMap;
use serde::Deserialize;
use tokio::fs::{copy, write};
use url::Url;

use stencila_codec_docx::extract_properties;
use stencila_schema::Primitive;

use crate::{base_url, client, process_response};

/// Response from the email attachments endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAttachmentsResponse {
    pub watch_id: String,
    pub attachments: Vec<EmailAttachment>,
}

/// An email attachment
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAttachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub download_url: String,
    pub expires_at: String,
}

/// Reference to email attachments for a watch
#[derive(Debug, Clone)]
pub struct EmailAttachmentsRef {
    /// The watch ID
    pub watch_id: String,
}

/// Parse an email attachments URL to extract the watch ID
///
/// Supports format: `https://api.stencila.cloud/v1/watches/{watch_id}/email/attachments`
pub fn parse_email_attachments_url(url: &Url) -> Result<EmailAttachmentsRef> {
    // Check host - support both production and custom API URLs
    let expected_host = Url::parse(&base_url())
        .ok()
        .and_then(|u| u.host_str().map(String::from));

    let url_host = url.host_str().map(String::from);

    if url_host != expected_host {
        bail!(
            "Not a Stencila Cloud URL: {url}. Expected host: {}",
            expected_host.unwrap_or_else(|| "api.stencila.cloud".to_string())
        );
    }

    // Parse path segments: ["v1", "watches", "{watch_id}", "email", "attachments"]
    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| eyre!("Invalid URL: no path"))?
        .collect();

    if segments.len() < 5
        || segments[0] != "v1"
        || segments[1] != "watches"
        || segments[3] != "email"
        || segments[4] != "attachments"
    {
        bail!(
            "Invalid email attachments URL format: {url}. Expected: {}/watches/{{watch_id}}/email/attachments",
            base_url()
        );
    }

    let watch_id = segments[2].to_string();

    Ok(EmailAttachmentsRef { watch_id })
}

/// Check if a URL matches the email attachments pattern
pub fn matches_url(url: &Url) -> bool {
    parse_email_attachments_url(url).is_ok()
}

/// Pull a DOCX from email attachments that matches the target path and repository
///
/// Fetches the attachments list, downloads DOCX files, and matches by path and repository properties.
///
/// The `target_path` is used to select the correct DOCX when multiple are attached.
/// The `repo_url` is used to verify the DOCX belongs to the correct repository.
#[tracing::instrument(skip(dest))]
pub async fn pull(
    url: &Url,
    dest: &Path,
    target_path: Option<&Path>,
    repo_url: Option<&str>,
) -> Result<()> {
    let attachments_ref = parse_email_attachments_url(url)?;

    // Get authenticated client
    let http_client = client().await?;

    // Fetch attachments list
    let attachments_url = format!(
        "{}/watches/{}/email/attachments",
        base_url(),
        attachments_ref.watch_id
    );

    let response = http_client.get(&attachments_url).send().await?;
    let attachments_response: EmailAttachmentsResponse = process_response(response).await?;

    // Filter for DOCX attachments
    let docx_attachments: Vec<_> = attachments_response
        .attachments
        .into_iter()
        .filter(|a| {
            a.filename.to_lowercase().ends_with(".docx")
                || a.content_type
                    == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        })
        .collect();

    if docx_attachments.is_empty() {
        bail!("No DOCX attachments found in the email");
    }

    // Download each DOCX and build a map of (path, repo) -> temp file
    // Using IndexMap to preserve insertion order while de-duplicating (last one wins)
    let mut path_to_docx: IndexMap<String, (Option<String>, tempfile::NamedTempFile)> =
        IndexMap::new();

    for attachment in &docx_attachments {
        // Get extension from filename for codec detection
        let extension = Path::new(&attachment.filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_else(|| ".docx".to_string());

        // Download to temp file with correct extension so codecs can detect format
        let temp_file = tempfile::Builder::new().suffix(&extension).tempfile()?;

        let response = http_client.get(&attachment.download_url).send().await?;

        if !response.status().is_success() {
            tracing::warn!(
                "Failed to download DOCX from {}: {}",
                attachment.download_url,
                response.status()
            );
            continue;
        }

        let bytes = response.bytes().await?.to_vec();
        write(temp_file.path(), &bytes).await?;

        // Extract path and repository properties from DOCX
        let properties = match extract_properties(temp_file.path()) {
            Ok((_, props)) => props,
            Err(e) => {
                tracing::warn!(
                    "Failed to extract properties from DOCX '{}': {e}",
                    attachment.filename
                );
                continue;
            }
        };

        let docx_path = properties.get("path").and_then(|p| match p {
            Primitive::String(s) => Some(s.clone()),
            _ => None,
        });

        let docx_repo = properties.get("repository").and_then(|p| match p {
            Primitive::String(s) => Some(s.clone()),
            _ => None,
        });

        if let Some(path_str) = docx_path {
            // Insert or overwrite - last one wins
            path_to_docx.insert(path_str, (docx_repo, temp_file));
        } else {
            tracing::warn!(
                "DOCX '{}' does not have a `path` property",
                attachment.filename
            );
        }
    }

    if path_to_docx.is_empty() {
        bail!(
            "No DOCX attachment contains a `path` property. Cannot determine path to target document."
        );
    }

    // Find matching DOCX based on target path and repository
    let matching = if let Some(target) = target_path {
        let target_str = target.to_string_lossy();

        // Find entries that match the target path and repository
        let mut matching_entry: Option<(&String, &tempfile::NamedTempFile)> = None;
        for (path_str, (docx_repo, temp_file)) in &path_to_docx {
            // Check path match
            let path_matches = path_str == target_str.as_ref()
                || target_str.ends_with(path_str.as_str())
                || path_str.ends_with(target_str.as_ref());

            if !path_matches {
                continue;
            }

            // Check repository match if repo_url is provided
            if let Some(expected_repo) = repo_url {
                match docx_repo {
                    Some(docx_repo_url) => {
                        // Normalize URLs for comparison (remove trailing slashes, .git suffix)
                        let normalize = |s: &str| {
                            s.trim_end_matches('/')
                                .trim_end_matches(".git")
                                .to_lowercase()
                        };
                        if normalize(docx_repo_url) != normalize(expected_repo) {
                            tracing::debug!(
                                "DOCX repository '{}' does not match expected '{}'",
                                docx_repo_url,
                                expected_repo
                            );
                            continue;
                        }
                    }
                    None => {
                        tracing::warn!(
                            "DOCX at path '{}' does not have a `repository` property",
                            path_str
                        );
                        continue;
                    }
                }
            }

            matching_entry = Some((path_str, temp_file));
            // Don't break - last one wins (IndexMap preserves insertion order)
        }

        matching_entry.map(|(_, tf)| tf)
    } else {
        // No target path - use first entry if there's only one, error otherwise
        if path_to_docx.len() == 1 {
            path_to_docx.values().next().map(|(_, tf)| tf)
        } else {
            let available_paths: Vec<&str> = path_to_docx.keys().map(|s| s.as_str()).collect();
            bail!(
                "Multiple DOCX attachments found with different paths: {}. Specify a target path to select one.",
                available_paths.join(", ")
            );
        }
    };

    let Some(matching) = matching else {
        let available_paths: Vec<&str> = path_to_docx.keys().map(|s| s.as_str()).collect();
        bail!(
            "No DOCX attachment matches the target path `{}`. Paths found in metadata of attachments: {}",
            target_path
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            available_paths.join(", ")
        );
    };

    copy(matching.path(), dest).await?;

    Ok(())
}

/// Pull all DOCX attachments and return them with their target paths
///
/// Fetches attachments once, extracts path metadata from each, and returns
/// a list of (target_path, temp_file) pairs. The caller is responsible for
/// converting/merging these files to their destinations.
///
/// The `repo_url` is used to filter attachments to only those matching the repository.
#[tracing::instrument]
pub async fn pull_all(
    url: &Url,
    repo_url: Option<&str>,
) -> Result<Vec<(PathBuf, tempfile::NamedTempFile)>> {
    let attachments_ref = parse_email_attachments_url(url)?;

    // Get authenticated client
    let http_client = client().await?;

    // Fetch attachments list (once)
    let attachments_url = format!(
        "{}/watches/{}/email/attachments",
        base_url(),
        attachments_ref.watch_id
    );

    let response = http_client.get(&attachments_url).send().await?;
    let attachments_response: EmailAttachmentsResponse = process_response(response).await?;

    // Filter for DOCX attachments
    let docx_attachments: Vec<_> = attachments_response
        .attachments
        .into_iter()
        .filter(|a| {
            a.filename.to_lowercase().ends_with(".docx")
                || a.content_type
                    == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        })
        .collect();

    if docx_attachments.is_empty() {
        bail!("No DOCX attachments found in the email");
    }

    // Collect files in IndexMap - last one wins for duplicate paths
    let mut path_to_file: IndexMap<String, tempfile::NamedTempFile> = IndexMap::new();

    for attachment in &docx_attachments {
        // Get extension from filename for codec detection
        let extension = Path::new(&attachment.filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_else(|| ".docx".to_string());

        // Download to temp file with correct extension so codecs can detect format
        let temp_file = tempfile::Builder::new().suffix(&extension).tempfile()?;

        let response = http_client.get(&attachment.download_url).send().await?;

        if !response.status().is_success() {
            tracing::warn!(
                "Failed to download DOCX from {}: {}",
                attachment.download_url,
                response.status()
            );
            continue;
        }

        let bytes = response.bytes().await?.to_vec();
        write(temp_file.path(), &bytes).await?;

        // Extract path and repository properties from DOCX
        let properties = match extract_properties(temp_file.path()) {
            Ok((_, props)) => props,
            Err(e) => {
                tracing::warn!(
                    "Failed to extract properties from DOCX '{}': {e}",
                    attachment.filename
                );
                continue;
            }
        };

        // Get path property
        let Some(path_str) = properties.get("path").and_then(|p| match p {
            Primitive::String(s) => Some(s.clone()),
            _ => None,
        }) else {
            tracing::warn!(
                "DOCX '{}' does not have a `path` property",
                attachment.filename
            );
            continue;
        };

        // Validate repository if provided
        if let Some(expected_repo) = repo_url {
            let docx_repo = properties.get("repository").and_then(|p| match p {
                Primitive::String(s) => Some(s.clone()),
                _ => None,
            });

            match docx_repo {
                Some(docx_repo_url) => {
                    // Normalize URLs for comparison (remove trailing slashes, .git suffix)
                    let normalize = |s: &str| {
                        s.trim_end_matches('/')
                            .trim_end_matches(".git")
                            .to_lowercase()
                    };
                    if normalize(&docx_repo_url) != normalize(expected_repo) {
                        tracing::debug!(
                            "DOCX repository '{}' does not match expected '{}', skipping",
                            docx_repo_url,
                            expected_repo
                        );
                        continue;
                    }
                }
                None => {
                    tracing::warn!(
                        "DOCX at path '{}' does not have a `repository` property, skipping",
                        path_str
                    );
                    continue;
                }
            }
        }

        // Insert or overwrite - last one wins for duplicate paths
        path_to_file.insert(path_str, temp_file);
    }

    if path_to_file.is_empty() {
        bail!("No DOCX attachments with valid path metadata found");
    }

    // Return (target_path, temp_file) pairs for caller to convert/merge
    let results: Vec<_> = path_to_file
        .into_iter()
        .map(|(path_str, temp_file)| (PathBuf::from(path_str), temp_file))
        .collect();

    Ok(results)
}

/// Get the last modified timestamp of email attachments
///
/// Returns the `last_remote_received_at` timestamp from the watch,
/// which indicates when an email was last received for this watch.
#[tracing::instrument]
pub async fn modified_at(url: &Url) -> Result<u64> {
    let attachments_ref = parse_email_attachments_url(url)?;

    // Fetch the watch details to get last_remote_received_at
    let watch = crate::watch::get_watch(&attachments_ref.watch_id).await?;

    // Parse last_remote_received_at as a timestamp
    // If not set, return 0
    let timestamp = watch
        .last_remote_received_at
        .and_then(|ts| {
            chrono::DateTime::parse_from_rfc3339(&ts)
                .ok()
                .map(|dt| dt.timestamp() as u64)
        })
        .unwrap_or(0);

    Ok(timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_email_attachments_url_valid() -> Result<()> {
        let url = Url::parse("https://api.stencila.cloud/v1/watches/wAbC12345/email/attachments")?;
        let result = parse_email_attachments_url(&url)?;

        assert_eq!(result.watch_id, "wAbC12345");
        Ok(())
    }

    #[test]
    fn test_parse_email_attachments_url_invalid_host() -> Result<()> {
        let url = Url::parse("https://example.com/v1/watches/wAbC12345/email/attachments")?;
        let result = parse_email_attachments_url(&url);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_email_attachments_url_invalid_path() -> Result<()> {
        let url = Url::parse("https://api.stencila.cloud/v1/watches/wAbC12345/other")?;
        let result = parse_email_attachments_url(&url);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_matches_url_valid() -> Result<()> {
        let url = Url::parse("https://api.stencila.cloud/v1/watches/wAbC12345/email/attachments")?;
        assert!(matches_url(&url));
        Ok(())
    }

    #[test]
    fn test_matches_url_invalid() -> Result<()> {
        let url = Url::parse("https://github.com/owner/repo/issues/123")?;
        assert!(!matches_url(&url));
        Ok(())
    }
}
