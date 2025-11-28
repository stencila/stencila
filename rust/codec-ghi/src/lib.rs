//! GitHub Issues remote provider for Stencila
//!
//! This codec provides pull functionality for retrieving DOCX attachments
//! from GitHub issue bodies or comments.

use std::{
    io::{IsTerminal, Write, stderr, stdin},
    path::Path,
};

use chrono::{DateTime, Utc};
use directories::UserDirs;
use eyre::{Result, bail, eyre};
use regex::Regex;
use reqwest::Client;
use tokio::fs::{copy, read, write};
use url::Url;

use stencila_cli_utils::message;
use stencila_codec::stencila_schema::Primitive;

/// Reference to a GitHub issue or comment
#[derive(Debug, Clone)]
pub struct GitHubIssueRef {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Issue number
    pub issue_number: u64,
    /// Optional comment ID (from #issuecomment-ID fragment)
    pub comment_id: Option<u64>,
}

/// Parse a GitHub issue URL to extract owner, repo, issue number, and optional comment ID
///
/// Supports formats:
/// - `https://github.com/owner/repo/issues/123`
/// - `https://github.com/owner/repo/issues/123#issuecomment-456`
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

    // Check for comment ID in fragment
    let comment_id = url.fragment().and_then(|fragment| {
        fragment
            .strip_prefix("issuecomment-")
            .and_then(|id| id.parse().ok())
    });

    Ok(GitHubIssueRef {
        owner,
        repo,
        issue_number,
        comment_id,
    })
}

/// Get GitHub API token for accessing a specific repository
///
/// Tries in order:
/// 1. GITHUB_TOKEN environment variable or keyring secret
/// 2. Stencila Cloud GitHub App installation token for the repository
/// 3. Stencila Cloud GitHub OAuth token (user's connected GitHub account)
async fn get_github_token(owner: &str, repo: &str) -> Result<String> {
    // First try local secret/env var
    if let Ok(token) = stencila_secrets::env_or_get(stencila_secrets::GITHUB_TOKEN) {
        return Ok(token);
    }

    // Try to get a repository-specific installation token from Stencila Cloud
    // This uses the Stencila GitHub App installation on the repository
    if let Ok(token) = stencila_cloud::get_repo_token(owner, repo).await {
        return Ok(token);
    }

    // Fall back to user's connected GitHub OAuth token
    stencila_cloud::get_token("github").await
}

/// Find all DOCX attachment URLs in a markdown body
///
/// GitHub file attachments appear as:
/// - `[filename.docx](https://github.com/user-attachments/assets/...)`
/// - `[filename.docx](https://github.com/owner/repo/files/...)`
pub fn find_docx_attachments(body: &str) -> Vec<String> {
    // Match markdown links ending in .docx (case-insensitive)
    let re = Regex::new(r"(?i)\[([^\]]*\.docx)\]\(([^)]+)\)").expect("valid regex");

    re.captures_iter(body)
        .filter_map(|cap| cap.get(2).map(|m| m.as_str().to_string()))
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

    // Extract filename from URL
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

/// Fetch issue or comment body from GitHub API
async fn fetch_github_content(
    issue_ref: &GitHubIssueRef,
    token: Option<&str>,
) -> Result<(String, DateTime<Utc>)> {
    let client = Client::new();

    let url = if let Some(comment_id) = issue_ref.comment_id {
        format!(
            "https://api.github.com/repos/{}/{}/issues/comments/{}",
            issue_ref.owner, issue_ref.repo, comment_id
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            issue_ref.owner, issue_ref.repo, issue_ref.issue_number
        )
    };

    let mut request = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "stencila");

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 | 403 => bail!(
                "Authentication required to access this repository. Set GITHUB_TOKEN or connect your GitHub account.\n\nError: {error_text}"
            ),
            404 => bail!(
                "GitHub issue not found (404). Check the URL is correct.\n\nError: {error_text}"
            ),
            429 => bail!("GitHub API rate limit exceeded. Try again later."),
            _ => bail!("Failed to fetch from GitHub ({status}): {error_text}"),
        }
    }

    // Parse response - structure is same for issues and comments
    let json: serde_json::Value = response.json().await?;

    let body = json
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let updated_at_str = json
        .get("updated_at")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre!("Missing updated_at field in response"))?;

    let updated_at = DateTime::parse_from_rfc3339(updated_at_str)
        .map_err(|e| eyre!("Failed to parse updated_at: {e}"))?
        .with_timezone(&Utc);

    Ok((body, updated_at))
}

/// Pull a DOCX from a GitHub issue/comment that matches the target path
///
/// When an issue has multiple DOCX attachments, this finds the one whose
/// embedded `path` property matches `target_path`. If multiple match,
/// the last one wins.
///
/// The `target_path` is used to select the correct DOCX when multiple
/// are attached. It should be the path to the local document being updated.
#[tracing::instrument(skip(dest))]
pub async fn pull(url: &Url, dest: &Path, target_path: Option<&Path>) -> Result<()> {
    let issue_ref = parse_github_issue_url(url)?;

    // Get token (optional for public repos)
    let token = get_github_token(&issue_ref.owner, &issue_ref.repo)
        .await
        .ok();

    // Fetch the issue/comment body
    let (body, _updated_at) = fetch_github_content(&issue_ref, token.as_deref()).await?;

    // Find all DOCX attachments
    let docx_urls = find_docx_attachments(&body);

    if docx_urls.is_empty() {
        bail!("No DOCX attachment found in the GitHub issue body");
    }

    // Download each DOCX and check its path property
    let client = Client::new();
    let mut matching_docx: Option<tempfile::NamedTempFile> = None;
    let mut available_paths: Vec<String> = Vec::new();

    for docx_url in &docx_urls {
        // Download to temp file
        let temp_file = tempfile::NamedTempFile::new()?;

        let mut request = client
            .get(docx_url)
            .header("User-Agent", "stencila")
            .header("Accept", "application/octet-stream");

        if let Some(ref token) = token {
            // GitHub user-attachments require "token" format, not "Bearer"
            request = request.header("Authorization", format!("token {token}"));
        }

        let response = request.send().await?;

        let bytes = if response.status() == reqwest::StatusCode::NOT_FOUND {
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

        if let Some(ref path_str) = docx_path {
            available_paths.push(path_str.clone());

            // Check if this matches our target path
            if let Some(target) = target_path {
                let target_str = target.to_string_lossy();
                if path_str == target_str.as_ref()
                    || target_str.ends_with(path_str)
                    || path_str.ends_with(target_str.as_ref())
                {
                    matching_docx = Some(temp_file);
                    // Don't break - last one wins
                }
            } else {
                // No target path specified - use first DOCX with path property
                if matching_docx.is_none() {
                    matching_docx = Some(temp_file);
                }
            }
        } else {
            tracing::warn!("DOCX at {docx_url} does not have a `path` property");
        }
    }

    // Copy matching DOCX to destination
    let Some(matching) = matching_docx else {
        if available_paths.is_empty() {
            bail!(
                "No DOCX attachment contains a `path` property. Cannot determine path to target document."
            );
        } else {
            bail!(
                "No DOCX attachment matches the target path `{}`. Paths found in metadata of attachments: {}",
                target_path
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
                available_paths.join(", ")
            );
        }
    };

    copy(matching.path(), dest).await?;

    Ok(())
}

/// Get the last modified timestamp of a GitHub issue/comment
#[tracing::instrument]
pub async fn modified_at(url: &Url) -> Result<u64> {
    let issue_ref = parse_github_issue_url(url)?;

    // Get token (optional for public repos)
    let token = get_github_token(&issue_ref.owner, &issue_ref.repo)
        .await
        .ok();

    // Fetch the issue/comment
    let (_body, updated_at) = fetch_github_content(&issue_ref, token.as_deref()).await?;

    Ok(updated_at.timestamp() as u64)
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
        assert!(result.comment_id.is_none());
        Ok(())
    }

    #[test]
    fn test_parse_github_issue_url_with_comment() -> Result<()> {
        let url =
            Url::parse("https://github.com/stencila/stencila/issues/2656#issuecomment-123456")?;
        let result = parse_github_issue_url(&url)?;

        assert_eq!(result.owner, "stencila");
        assert_eq!(result.repo, "stencila");
        assert_eq!(result.issue_number, 2656);
        assert_eq!(result.comment_id, Some(123456));
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
[report.docx](https://github.com/user-attachments/assets/abc123)

And another:
[data.DOCX](https://github.com/stencila/stencila/files/456789/data.DOCX)

Not a docx: [readme.md](https://example.com/readme.md)
        "#;

        let attachments = find_docx_attachments(body);
        assert_eq!(attachments.len(), 2);
        assert_eq!(
            attachments[0],
            "https://github.com/user-attachments/assets/abc123"
        );
        assert_eq!(
            attachments[1],
            "https://github.com/stencila/stencila/files/456789/data.DOCX"
        );
    }

    #[test]
    fn test_find_docx_attachments_empty() {
        let body = "No attachments here, just plain text.";
        let attachments = find_docx_attachments(body);
        assert!(attachments.is_empty());
    }
}
