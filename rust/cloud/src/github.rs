use eyre::{Result, bail};
use reqwest::Client;

use crate::{base_url, client, tokens};

tokens::connection_token_error!(
    /// Custom error types for GitHub integration.
    enum GitHubTokenError,
    "GitHub"
);

/// Get a GitHub access token without retry using an explicit Cloud API client.
pub async fn get_token_once_with_client(client: &Client) -> Result<String> {
    tokens::get_once_with_client(client, "github")
        .await
        .map_err(GitHubTokenError::from)
        .map_err(Into::into)
}

/// Get a GitHub App installation token for a specific repository
///
/// This function calls the Stencila Cloud API to retrieve an installation token
/// that provides access to a specific repository where the Stencila GitHub App
/// is installed. This is different from user OAuth tokens - it uses the App's
/// installation permissions.
///
/// # Arguments
///
/// * `owner` - The repository owner (user or organization)
/// * `repo` - The repository name
///
/// Returns an error if the app is not installed on the repository or if
/// the user doesn't have access.
pub async fn get_repo_token(owner: &str, repo: &str) -> Result<String> {
    let client = client().await?;

    get_repo_token_with_client(&client, owner, repo).await
}

/// Get a GitHub App installation token using an explicit Cloud API client.
pub async fn get_repo_token_with_client(
    client: &Client,
    owner: &str,
    repo: &str,
) -> Result<String> {
    // Call the repository token endpoint
    let url = format!("{}/connections/github/token/{}/{}", base_url(), owner, repo);
    let response = client.get(&url).send().await?;

    let status = response.status();

    match status.as_u16() {
        200 => {
            // Success - parse and return access token
            let token_response = response
                .json::<tokens::TokenResponse>()
                .await
                .map_err(|e| eyre::eyre!("Failed to parse response: {e}"))?;
            Ok(token_response.access_token)
        }
        404 => {
            bail!(
                "Stencila GitHub App is not installed on {owner}/{repo}. \
                 Install it at https://github.com/apps/stencila/installations/new"
            )
        }
        403 => {
            bail!(
                "You don't have access to {owner}/{repo} or the Stencila GitHub App \
                 doesn't have permission to access this repository."
            )
        }
        _ => {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("HTTP error: {status}"));
            bail!("Failed to get repository token: {error_msg}")
        }
    }
}

/// Get GitHub access token with retry for connection flow
#[allow(clippy::print_stderr)]
pub async fn get_token_with_retry() -> Result<String> {
    tokens::get_with_retry("GitHub", "github").await
}
