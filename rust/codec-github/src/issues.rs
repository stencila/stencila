//! GitHub Issues remote provider for Stencila
//!
//! This module provides pull functionality for retrieving DOCX attachments
//! from GitHub issue bodies or comments.

use std::{
    io::{IsTerminal, Write, stderr, stdin},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use chrono::{DateTime, Utc};
use directories::UserDirs;
use indexmap::IndexMap;
use regex::Regex;
use reqwest::{StatusCode, header::HeaderValue};
use tokio::fs::{copy, read, write};
use url::Url;

use stencila_cli_utils::message;
use stencila_codec::{
    eyre::{Result, bail, eyre},
    stencila_schema::Primitive,
};

use crate::client::{CLIENT, apply_rate_limiting, get_token};

/// Regex for matching DOCX markdown links in GitHub issue bodies
///
/// Matches markdown links `[filename.docx](url)` where the URL is a GitHub attachment:
/// - `https://github.com/user-attachments/files/...` (documents)
/// - `https://github.com/user-attachments/assets/...` (assets)
/// - `https://github.com/owner/repo/files/...` (older format)
static DOCX_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\[([^\]]+\.docx)\]\((https://(?:github\.com/user-attachments/(?:assets|files)/[a-f0-9-]+(?:/[^)\s]+)?|github\.com/[^/]+/[^/]+/files/[^)]+))\)"
    ).expect("hardcoded regex should be valid")
});

/// Pull a DOCX from a GitHub issue that matches the target path
///
/// Fetches the issue body and all comments, extracts DOCX attachments from all of them,
/// and de-duplicates by target path (last one wins).
///
/// The `target_path` is used to select the correct DOCX when multiple
/// are attached. It should be the path to the local document being updated.
#[tracing::instrument(skip(dest))]
pub async fn pull(url: &Url, dest: &Path, target_path: Option<&Path>) -> Result<()> {
    let issue_ref = parse_github_issue_url(url)?;

    // Get token (optional for public repos)
    let token = get_token(Some(&issue_ref.owner), Some(&issue_ref.repo)).await;

    // Fetch issue body and all comments
    let all_content = fetch_all_issue_content(&issue_ref, token.as_deref()).await?;

    // Find all DOCX attachments from issue body and all comments
    let mut all_attachments = Vec::new();
    for content in &all_content {
        all_attachments.extend(find_docx_attachments(&content.body));
    }

    if all_attachments.is_empty() {
        bail!("No DOCX attachments found in the GitHub issue body or comments");
    }

    // Download each DOCX and build a map of path -> temp file
    // Using IndexMap to preserve insertion order while de-duplicating (last one wins)
    let mut path_to_docx: IndexMap<String, tempfile::NamedTempFile> = IndexMap::new();

    for (filename, docx_url) in &all_attachments {
        // Get extension from filename for codec detection
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_else(|| ".docx".to_string());

        // Download to temp file with correct extension so codecs can detect format
        let temp_file = tempfile::Builder::new().suffix(&extension).tempfile()?;

        // Note: DOCX downloads use different headers than API requests:
        // - Accept: octet-stream for binary content
        // - Authorization: "token" format (not "Bearer") for user-attachments
        let mut request = CLIENT
            .get(docx_url)
            .header("Accept", "application/octet-stream");

        if let Some(ref token) = token {
            request = request.header("Authorization", format!("token {token}"));
        }

        let response = request.send().await?;

        let bytes = if response.status() == StatusCode::NOT_FOUND {
            // Likely a private repo attachment - try browser-assisted download
            match browser_assisted_download(docx_url).await {
                Ok(Some(bytes)) => bytes,
                Ok(None) => continue, // User skipped
                Err(e) => {
                    tracing::warn!("Browser-assisted download failed: {e}");
                    continue;
                }
            }
        } else if !response.status().is_success() {
            tracing::warn!(
                "Failed to download DOCX from {docx_url}: {}",
                response.status()
            );
            continue;
        } else {
            response.bytes().await?.to_vec()
        };

        write(temp_file.path(), &bytes).await?;

        // Extract path property from DOCX
        let properties = match stencila_codec_docx::extract_properties(temp_file.path()) {
            Ok((_, props)) => props,
            Err(e) => {
                tracing::warn!("Failed to extract properties from DOCX at {docx_url}: {e}");
                continue;
            }
        };

        let docx_path = properties.get("path").and_then(|p| match p {
            Primitive::String(s) => Some(s.clone()),
            _ => None,
        });

        if let Some(path_str) = docx_path {
            // Insert or overwrite - last one wins
            path_to_docx.insert(path_str, temp_file);
        } else {
            tracing::warn!("DOCX at {docx_url} does not have a `path` property");
        }
    }

    if path_to_docx.is_empty() {
        bail!(
            "No DOCX attachment contains a `path` property. Cannot determine path to target document."
        );
    }

    // Find matching DOCX based on target path
    let matching = if let Some(target) = target_path {
        let target_str = target.to_string_lossy();

        // Find entries that match the target path
        let mut matching_entry: Option<(&String, &tempfile::NamedTempFile)> = None;
        for (path_str, temp_file) in &path_to_docx {
            if path_str == target_str.as_ref()
                || target_str.ends_with(path_str.as_str())
                || path_str.ends_with(target_str.as_ref())
            {
                matching_entry = Some((path_str, temp_file));
                // Don't break - last one wins (IndexMap preserves insertion order)
            }
        }

        matching_entry.map(|(_, tf)| tf)
    } else {
        // No target path - use first entry if there's only one, error otherwise
        if path_to_docx.len() == 1 {
            path_to_docx.values().next()
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
/// Fetches the issue body and all comments, extracts DOCX attachments,
/// and returns a list of (target_path, temp_file) pairs. The caller is
/// responsible for converting/merging these files to their destinations.
#[tracing::instrument]
pub async fn pull_all(url: &Url) -> Result<Vec<(PathBuf, tempfile::NamedTempFile)>> {
    let issue_ref = parse_github_issue_url(url)?;

    // Get token (optional for public repos)
    let token = get_token(Some(&issue_ref.owner), Some(&issue_ref.repo)).await;

    // Fetch issue body and all comments
    let all_content = fetch_all_issue_content(&issue_ref, token.as_deref()).await?;

    // Find all DOCX attachments from issue body and all comments
    let mut all_attachments = Vec::new();
    for content in &all_content {
        all_attachments.extend(find_docx_attachments(&content.body));
    }

    if all_attachments.is_empty() {
        bail!("No DOCX attachments found in the GitHub issue body or comments");
    }

    // Collect files in IndexMap - last one wins for duplicate paths
    let mut path_to_file: IndexMap<String, tempfile::NamedTempFile> = IndexMap::new();

    for (filename, docx_url) in &all_attachments {
        // Get extension from filename for codec detection
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_else(|| ".docx".to_string());

        // Download to temp file with correct extension so codecs can detect format
        let temp_file = tempfile::Builder::new().suffix(&extension).tempfile()?;

        // Note: DOCX downloads use different headers than API requests:
        // - Accept: octet-stream for binary content
        // - Authorization: "token" format (not "Bearer") for user-attachments
        let mut request = CLIENT
            .get(docx_url)
            .header("Accept", "application/octet-stream");

        if let Some(ref token) = token {
            request = request.header("Authorization", format!("token {token}"));
        }

        let response = request.send().await?;

        let bytes = if response.status() == StatusCode::NOT_FOUND {
            // Likely a private repo attachment - try browser-assisted download
            match browser_assisted_download(docx_url).await {
                Ok(Some(bytes)) => bytes,
                Ok(None) => continue, // User skipped
                Err(e) => {
                    tracing::warn!("Browser-assisted download failed: {e}");
                    continue;
                }
            }
        } else if !response.status().is_success() {
            tracing::warn!(
                "Failed to download DOCX from {docx_url}: {}",
                response.status()
            );
            continue;
        } else {
            response.bytes().await?.to_vec()
        };

        write(temp_file.path(), &bytes).await?;

        // Extract path property from DOCX
        let properties = match stencila_codec_docx::extract_properties(temp_file.path()) {
            Ok((_, props)) => props,
            Err(e) => {
                tracing::warn!("Failed to extract properties from DOCX at {docx_url}: {e}");
                continue;
            }
        };

        // Get path property
        let Some(path_str) = properties.get("path").and_then(|p| match p {
            Primitive::String(s) => Some(s.clone()),
            _ => None,
        }) else {
            tracing::warn!("DOCX at {docx_url} does not have a `path` property");
            continue;
        };

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

/// Content extracted from an issue body or comment
struct IssueContent {
    /// The markdown body text
    body: String,
    /// When this content was last updated
    updated_at: DateTime<Utc>,
}

/// Reference to a GitHub issue
#[derive(Debug, Clone)]
pub struct GitHubIssueRef {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Issue number
    pub issue_number: u64,
}

/// Parse a GitHub issue URL to extract owner, repo, and issue number
///
/// Supports format: `https://github.com/owner/repo/issues/123`
///
/// Note: Any `#issuecomment-*` fragment is ignored since we fetch all comments.
pub fn parse_github_issue_url(url: &Url) -> Result<GitHubIssueRef> {
    // Check host
    if url.host_str() != Some("github.com") {
        bail!("Not a GitHub URL: {url}");
    }

    // Parse path segments: ["owner", "repo", "issues", "123"]
    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| eyre!("Invalid URL: no path"))?
        .collect();

    if segments.len() < 4 || segments[2] != "issues" {
        bail!(
            "Invalid GitHub issue URL format: {url}. Expected: https://github.com/owner/repo/issues/number"
        );
    }

    let owner = segments[0].to_string();
    let repo = segments[1].to_string();
    let issue_number: u64 = segments[3]
        .parse()
        .map_err(|_| eyre!("Invalid issue number: {}", segments[3]))?;

    Ok(GitHubIssueRef {
        owner,
        repo,
        issue_number,
    })
}

/// Build a GitHub API request with common headers
fn build_github_api_request(url: &str, token: Option<&str>) -> reqwest::RequestBuilder {
    let mut request = CLIENT.get(url).header(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    request
}

/// Handle GitHub API error responses and convert to appropriate error messages
fn github_api_error(status: StatusCode, error_text: &str) -> stencila_codec::eyre::Report {
    match status.as_u16() {
        401 | 403 => eyre!(
            "Authentication required to access this repository. Set GITHUB_TOKEN or connect your GitHub account.\n\nError: {error_text}"
        ),
        404 => {
            eyre!("GitHub issue not found (404). Check the URL is correct.\n\nError: {error_text}")
        }
        429 => eyre!("GitHub API rate limit exceeded. Try again later."),
        _ => eyre!("Failed to fetch from GitHub ({status}): {error_text}"),
    }
}

/// Parse updated_at timestamp from a GitHub API JSON response
fn parse_updated_at(json: &serde_json::Value) -> Result<DateTime<Utc>> {
    let updated_at_str = json
        .get("updated_at")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre!("Missing updated_at field in response"))?;

    DateTime::parse_from_rfc3339(updated_at_str)
        .map_err(|e| eyre!("Failed to parse updated_at: {e}"))
        .map(|dt| dt.with_timezone(&Utc))
}

/// Fetch issue body and all comments
///
/// Returns all content in order: issue body first, then comments chronologically.
async fn fetch_all_issue_content(
    issue_ref: &GitHubIssueRef,
    token: Option<&str>,
) -> Result<Vec<IssueContent>> {
    let issue_body = fetch_issue_body(issue_ref, token).await?;
    let comments = fetch_issue_comments(issue_ref, token).await?;

    let mut all_content = Vec::with_capacity(1 + comments.len());
    all_content.push(issue_body);
    all_content.extend(comments);

    Ok(all_content)
}

/// Fetch issue body from GitHub API
async fn fetch_issue_body(issue_ref: &GitHubIssueRef, token: Option<&str>) -> Result<IssueContent> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}",
        issue_ref.owner, issue_ref.repo, issue_ref.issue_number
    );

    apply_rate_limiting(&url, token.is_some()).await?;

    let response = build_github_api_request(&url, token).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!(github_api_error(status, &error_text));
    }

    let json: serde_json::Value = response.json().await?;

    let body = json
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let updated_at = parse_updated_at(&json)?;

    Ok(IssueContent { body, updated_at })
}

/// Fetch all comments for an issue from GitHub API
///
/// Returns comments in chronological order (oldest first).
/// Handles pagination automatically.
async fn fetch_issue_comments(
    issue_ref: &GitHubIssueRef,
    token: Option<&str>,
) -> Result<Vec<IssueContent>> {
    let mut all_comments = Vec::new();
    let mut page = 1u32;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments?per_page=100&page={}",
            issue_ref.owner, issue_ref.repo, issue_ref.issue_number, page
        );

        apply_rate_limiting(&url, token.is_some()).await?;

        let response = build_github_api_request(&url, token).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            bail!(github_api_error(status, &error_text));
        }

        let json: serde_json::Value = response.json().await?;
        let comments = json
            .as_array()
            .ok_or_else(|| eyre!("Expected array of comments from GitHub API"))?;

        if comments.is_empty() {
            break;
        }

        for comment in comments {
            let body = comment
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let updated_at = parse_updated_at(comment)?;

            all_comments.push(IssueContent { body, updated_at });
        }

        // If we got fewer than 100 comments, we've reached the last page
        if comments.len() < 100 {
            break;
        }

        page += 1;
    }

    Ok(all_comments)
}

/// Find all DOCX attachments in a markdown body
///
/// Returns (filename, url) pairs for each attachment.
///
/// GitHub file attachments appear as:
/// - `[filename.docx](https://github.com/user-attachments/assets/...)`
/// - `[filename.docx](https://github.com/owner/repo/files/...)`
pub fn find_docx_attachments(body: &str) -> Vec<(String, String)> {
    DOCX_LINK_RE
        .captures_iter(body)
        .filter_map(|cap| {
            let filename = cap.get(1)?.as_str().to_string();
            let url = cap.get(2)?.as_str().to_string();
            Some((filename, url))
        })
        .collect()
}

/// Helper function for browser-assisted download
///
/// GitHub user-attachments URLs require session-based authentication (browser cookies).
/// Neither API tokens nor GitHub App installation tokens can access these attachments.
/// This is a known GitHub limitation:
/// - <https://github.com/orgs/community/discussions/162417>
/// - <https://stackoverflow.com/questions/78361554/download-github-issue-attachments-from-private-repos>
#[allow(clippy::print_stderr)]
async fn browser_assisted_download(docx_url: &str) -> Result<Option<Vec<u8>>> {
    message!("⚠️ Cannot download attachment directly, likely a private repo.");

    let answer = stencila_ask::ask(&format!("Open browser to download `{docx_url}`?")).await?;
    if answer.is_no_or_cancel() {
        bail!("Aborted, GitHub issue attachment not accessible");
    }

    if let Err(e) = webbrowser::open(docx_url) {
        message!("⚠️ Failed to open browser: {e}. Please open manually: {docx_url}");
    }

    let answer = stencila_ask::ask("Have you downloaded the file?").await?;
    if answer.is_no_or_cancel() {
        bail!("Aborted, GitHub issue attachment not downloaded");
    }

    // Extract filename from URL (rsplit always returns at least one element)
    let filename = docx_url.rsplit('/').next().unwrap_or("attachment.docx");

    // Try Downloads folder first
    if let Some(user_dirs) = UserDirs::new()
        && let Some(downloads_dir) = user_dirs.download_dir()
    {
        let downloaded_path = downloads_dir.join(filename);
        if downloaded_path.exists() {
            message!("✅ Found file in `Downloads` folder.");
            return Ok(Some(read(&downloaded_path).await?));
        }
    }

    // Not found in Downloads - prompt for path
    message!("⚠️ Could not find `{filename}` in `Downloads` folder.");

    // Check if stdin is a TTY for path input
    if !stdin().is_terminal() {
        message!("⚠️ Non-interactive environment. Cannot prompt for file path.");
        return Ok(None);
    }

    eprint!("   Enter the full path to the downloaded file (or press Enter to skip): ");
    stderr().flush()?;

    let mut path_input = String::new();
    stdin().read_line(&mut path_input)?;
    let path = path_input.trim();

    if path.is_empty() {
        return Ok(None);
    }

    let path = std::path::Path::new(path);
    if path.exists() {
        Ok(Some(read(path).await?))
    } else {
        message!("⚠️ File not found: `{}`", path.display());
        Ok(None)
    }
}

/// Get the last modified timestamp of a GitHub issue
///
/// Returns the maximum of the issue's `updated_at` and all comments' `updated_at`.
/// This ensures we detect when any part of the issue thread has been updated.
#[tracing::instrument]
pub async fn modified_at(url: &Url) -> Result<u64> {
    let issue_ref = parse_github_issue_url(url)?;

    // Get token (optional for public repos)
    let token = get_token(Some(&issue_ref.owner), Some(&issue_ref.repo)).await;

    // Fetch issue body and all comments
    let all_content = fetch_all_issue_content(&issue_ref, token.as_deref()).await?;

    // Find the maximum updated_at across all content
    let max_updated_at = all_content
        .iter()
        .map(|c| c.updated_at)
        .max()
        .ok_or_else(|| eyre!("No content found in issue"))?;

    Ok(max_updated_at.timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_issue_url_basic() -> Result<()> {
        let url = Url::parse("https://github.com/stencila/stencila/issues/2656")?;
        let result = parse_github_issue_url(&url)?;

        assert_eq!(result.owner, "stencila");
        assert_eq!(result.repo, "stencila");
        assert_eq!(result.issue_number, 2656);
        Ok(())
    }

    #[test]
    fn test_parse_github_issue_url_with_comment_fragment_ignored() -> Result<()> {
        // Comment fragments are now ignored - we always fetch all comments
        let url =
            Url::parse("https://github.com/stencila/stencila/issues/2656#issuecomment-123456")?;
        let result = parse_github_issue_url(&url)?;

        assert_eq!(result.owner, "stencila");
        assert_eq!(result.repo, "stencila");
        assert_eq!(result.issue_number, 2656);
        // No comment_id field - we fetch all comments
        Ok(())
    }

    #[test]
    fn test_parse_github_issue_url_invalid_host() -> Result<()> {
        let url = Url::parse("https://gitlab.com/owner/repo/issues/123")?;
        let result = parse_github_issue_url(&url);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_github_issue_url_invalid_path() -> Result<()> {
        let url = Url::parse("https://github.com/owner/repo/pulls/123")?;
        let result = parse_github_issue_url(&url);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_find_docx_attachments() {
        let body = r#"
Here is the document:
[report.docx](https://github.com/user-attachments/assets/a1b2c3d4-e5f6-7890-abcd-ef1234567890)

And another using files path:
[data.DOCX](https://github.com/user-attachments/files/a1b2c3d4-e5f6-7890-abcd-ef1234567890/data.DOCX)

Older format with owner/repo/files:
[old.docx](https://github.com/stencila/stencila/files/456789/old.docx)

Not a docx: [readme.md](https://example.com/readme.md)
Not a GitHub URL: [other.docx](https://example.com/other.docx)
        "#;

        let attachments = find_docx_attachments(body);
        assert_eq!(attachments.len(), 3);
        assert_eq!(
            attachments[0],
            (
                "report.docx".to_string(),
                "https://github.com/user-attachments/assets/a1b2c3d4-e5f6-7890-abcd-ef1234567890"
                    .to_string()
            )
        );
        assert_eq!(
            attachments[1],
            (
                "data.DOCX".to_string(),
                "https://github.com/user-attachments/files/a1b2c3d4-e5f6-7890-abcd-ef1234567890/data.DOCX"
                    .to_string()
            )
        );
        assert_eq!(
            attachments[2],
            (
                "old.docx".to_string(),
                "https://github.com/stencila/stencila/files/456789/old.docx".to_string()
            )
        );
    }

    #[test]
    fn test_find_docx_attachments_empty() {
        let body = "No attachments here, just plain text.";
        let attachments = find_docx_attachments(body);
        assert!(attachments.is_empty());
    }
}
