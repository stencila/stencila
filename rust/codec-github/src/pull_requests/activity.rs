//! GitHub pull request remote provider helpers for Stencila.
//!
//! This module currently provides read-side metadata used to describe the state
//! of a GitHub pull request remote. Unlike issue remotes, GitHub pull request
//! remotes are push-only in Stencila, so the main need here is to detect whether
//! the remote review surface has changed since the last push.

use chrono::{DateTime, Utc};
use reqwest::{StatusCode, header::HeaderValue};
use url::Url;

use stencila_codec::eyre::{Result, bail, eyre};

use crate::client::{CLIENT, GitHubAuthPolicy, apply_rate_limiting, get_token};

/// Reference to a GitHub pull request.
#[derive(Debug, Clone)]
pub struct GitHubPullRequestRef {
    /// Repository owner.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Pull request number.
    pub pull_number: u64,
}

/// Parse a GitHub pull request URL to extract owner, repo, and pull request number.
///
/// Supports format: `https://github.com/owner/repo/pull/123`.
pub fn parse_github_pull_request_url(url: &Url) -> Result<GitHubPullRequestRef> {
    if url.host_str() != Some("github.com") {
        bail!("Not a GitHub URL: {url}");
    }

    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| eyre!("Invalid URL: no path"))?
        .collect();

    if segments.len() < 4 || segments[2] != "pull" {
        bail!(
            "Invalid GitHub pull request URL format: {url}. Expected: https://github.com/owner/repo/pull/number"
        );
    }

    let owner = segments[0].to_string();
    let repo = segments[1].to_string();
    let pull_number: u64 = segments[3]
        .parse()
        .map_err(|_| eyre!("Invalid pull request number: {}", segments[3]))?;

    Ok(GitHubPullRequestRef {
        owner,
        repo,
        pull_number,
    })
}

/// Build a GitHub API request with common headers.
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

/// Handle GitHub API error responses and convert to appropriate error messages.
fn github_api_error(status: StatusCode, error_text: &str) -> stencila_codec::eyre::Report {
    match status.as_u16() {
        401 | 403 => eyre!(
            "Authentication required to access this repository. Set GITHUB_TOKEN or connect your GitHub account.\n\nError: {error_text}"
        ),
        404 => eyre!(
            "GitHub pull request not found (404). Check the URL is correct.\n\nError: {error_text}"
        ),
        429 => eyre!("GitHub API rate limit exceeded. Try again later."),
        _ => eyre!("Failed to fetch from GitHub ({status}): {error_text}"),
    }
}

/// Parse an RFC3339 timestamp field from a GitHub API JSON response.
fn parse_timestamp_field(json: &serde_json::Value, field: &str) -> Result<DateTime<Utc>> {
    let timestamp = json
        .get(field)
        .and_then(|value| value.as_str())
        .ok_or_else(|| eyre!("Missing {field} field in response"))?;

    DateTime::parse_from_rfc3339(timestamp)
        .map_err(|error| eyre!("Failed to parse {field}: {error}"))
        .map(|datetime| datetime.with_timezone(&Utc))
}

/// Fetch the pull request's own `updated_at` timestamp.
async fn fetch_pull_request_updated_at(
    pr_ref: &GitHubPullRequestRef,
    token: Option<&str>,
) -> Result<DateTime<Utc>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        pr_ref.owner, pr_ref.repo, pr_ref.pull_number
    );

    apply_rate_limiting(&url, token.is_some()).await?;

    let response = build_github_api_request(&url, token).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!(github_api_error(status, &error_text));
    }

    let json: serde_json::Value = response.json().await?;
    parse_timestamp_field(&json, "updated_at")
}

/// Fetch all review comment `updated_at` timestamps for a pull request.
async fn fetch_review_comment_timestamps(
    pr_ref: &GitHubPullRequestRef,
    token: Option<&str>,
) -> Result<Vec<DateTime<Utc>>> {
    let mut timestamps = Vec::new();
    let mut page = 1u32;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/comments?per_page=100&page={}",
            pr_ref.owner, pr_ref.repo, pr_ref.pull_number, page
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
            .ok_or_else(|| eyre!("Expected array of pull request comments from GitHub API"))?;

        if comments.is_empty() {
            break;
        }

        for comment in comments {
            timestamps.push(parse_timestamp_field(comment, "updated_at")?);
        }

        if comments.len() < 100 {
            break;
        }

        page += 1;
    }

    Ok(timestamps)
}

/// Fetch all review submission timestamps for a pull request.
async fn fetch_review_submission_timestamps(
    pr_ref: &GitHubPullRequestRef,
    token: Option<&str>,
) -> Result<Vec<DateTime<Utc>>> {
    let mut timestamps = Vec::new();
    let mut page = 1u32;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews?per_page=100&page={}",
            pr_ref.owner, pr_ref.repo, pr_ref.pull_number, page
        );

        apply_rate_limiting(&url, token.is_some()).await?;

        let response = build_github_api_request(&url, token).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            bail!(github_api_error(status, &error_text));
        }

        let json: serde_json::Value = response.json().await?;
        let reviews = json
            .as_array()
            .ok_or_else(|| eyre!("Expected array of pull request reviews from GitHub API"))?;

        if reviews.is_empty() {
            break;
        }

        for review in reviews {
            if review
                .get("submitted_at")
                .and_then(|value| value.as_str())
                .is_some()
            {
                timestamps.push(parse_timestamp_field(review, "submitted_at")?);
            } else {
                timestamps.push(parse_timestamp_field(review, "updated_at")?);
            }
        }

        if reviews.len() < 100 {
            break;
        }

        page += 1;
    }

    Ok(timestamps)
}

/// Fetch all issue-comment timestamps for a pull request conversation thread.
async fn fetch_issue_comment_timestamps(
    pr_ref: &GitHubPullRequestRef,
    token: Option<&str>,
) -> Result<Vec<DateTime<Utc>>> {
    let mut timestamps = Vec::new();
    let mut page = 1u32;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments?per_page=100&page={}",
            pr_ref.owner, pr_ref.repo, pr_ref.pull_number, page
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
            .ok_or_else(|| eyre!("Expected array of issue comments from GitHub API"))?;

        if comments.is_empty() {
            break;
        }

        for comment in comments {
            timestamps.push(parse_timestamp_field(comment, "updated_at")?);
        }

        if comments.len() < 100 {
            break;
        }

        page += 1;
    }

    Ok(timestamps)
}

/// Get the last meaningful activity timestamp of a GitHub pull request remote.
///
/// Returns the maximum timestamp across the pull request's own `updated_at`,
/// inline review comments, review submissions, and conversation comments.
/// This approximates when the remote review surface last changed.
#[tracing::instrument]
pub async fn modified_at(url: &Url) -> Result<u64> {
    let pr_ref = parse_github_pull_request_url(url)?;

    let token = get_token(
        Some(&pr_ref.owner),
        Some(&pr_ref.repo),
        GitHubAuthPolicy::PreferRepoInstallation,
    )
    .await;

    let pr_updated_at = fetch_pull_request_updated_at(&pr_ref, token.as_deref()).await?;
    let review_comment_timestamps =
        fetch_review_comment_timestamps(&pr_ref, token.as_deref()).await?;
    let review_submission_timestamps =
        fetch_review_submission_timestamps(&pr_ref, token.as_deref()).await?;
    let issue_comment_timestamps =
        fetch_issue_comment_timestamps(&pr_ref, token.as_deref()).await?;

    let max_timestamp = std::iter::once(pr_updated_at)
        .chain(review_comment_timestamps)
        .chain(review_submission_timestamps)
        .chain(issue_comment_timestamps)
        .max()
        .ok_or_else(|| eyre!("No content found in pull request"))?;

    Ok(max_timestamp.timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_pull_request_url_basic() -> Result<()> {
        let url = Url::parse("https://github.com/stencila/stencila/pull/123")?;
        let result = parse_github_pull_request_url(&url)?;

        assert_eq!(result.owner, "stencila");
        assert_eq!(result.repo, "stencila");
        assert_eq!(result.pull_number, 123);
        Ok(())
    }

    #[test]
    fn test_parse_github_pull_request_url_invalid_host() -> Result<()> {
        let url = Url::parse("https://gitlab.com/owner/repo/pull/123")?;
        let result = parse_github_pull_request_url(&url);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_github_pull_request_url_invalid_path() -> Result<()> {
        let url = Url::parse("https://github.com/stencila/stencila/issues/123")?;
        let result = parse_github_pull_request_url(&url);
        assert!(result.is_err());
        Ok(())
    }
}
