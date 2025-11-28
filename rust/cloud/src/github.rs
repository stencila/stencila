use std::{fmt, io::Write};

use eyre::{Result, bail};
use serde::Deserialize;

use crate::{ErrorResponse, base_url, client};

/// Response from the GitHub token endpoint when successful
#[derive(Deserialize)]
#[allow(dead_code)]
struct GitHubTokenResponse {
    access_token: String,
}

/// Custom error types for GitHub integration
#[derive(Debug)]
enum GitHubTokenError {
    NotLinked { connect_url: Option<String> },
    RefreshFailed { connect_url: Option<String> },
    JsonParsing(String),
    Other(String),
}

impl fmt::Display for GitHubTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitHubTokenError::NotLinked { connect_url } => {
                write!(
                    f,
                    "GitHub account not connected. Connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            GitHubTokenError::RefreshFailed { connect_url } => {
                write!(
                    f,
                    "Failed to refresh GitHub token. Re-connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            GitHubTokenError::JsonParsing(msg) => write!(f, "Failed to parse response: {msg}"),
            GitHubTokenError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GitHubTokenError {}

/// Get a GitHub access token from Stencila Cloud
///
/// This function calls the Stencila Cloud API to retrieve a GitHub access token
/// for the authenticated user. The user must have connected their GitHub account
/// to their Stencila account.
///
/// Returns a `GitHubTokenError` if the account is not connected or token refresh fails.
async fn get_token() -> Result<String, GitHubTokenError> {
    // Get authenticated client
    let client = match client().await {
        Ok(client) => client,
        Err(error) => return Err(GitHubTokenError::Other(error.to_string())),
    };

    // Call the GitHub token endpoint
    let url = format!("{}/connections/github/token", base_url());
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(error) => return Err(GitHubTokenError::Other(format!("Network error: {error}"))),
    };

    let status = response.status();

    // Handle different status codes
    match status.as_u16() {
        200 => {
            // Success - parse and return access token
            let token_response = response
                .json::<GitHubTokenResponse>()
                .await
                .map_err(|e| GitHubTokenError::JsonParsing(e.to_string()))?;
            Ok(token_response.access_token)
        }
        422 => {
            // GitHub account not connected
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| GitHubTokenError::JsonParsing(e.to_string()))?;
            Err(GitHubTokenError::NotLinked {
                connect_url: error_response.url,
            })
        }
        500 => {
            // Token refresh failed
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| GitHubTokenError::JsonParsing(e.to_string()))?;
            Err(GitHubTokenError::RefreshFailed {
                connect_url: error_response.url,
            })
        }
        _ => {
            // Other error
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("HTTP error: {status}"));
            Err(GitHubTokenError::Other(error_msg))
        }
    }
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
    // Get authenticated client
    let client = client().await?;

    // Call the repository token endpoint
    let url = format!("{}/connections/github/token/{}/{}", base_url(), owner, repo);
    let response = client.get(&url).send().await?;

    let status = response.status();

    match status.as_u16() {
        200 => {
            // Success - parse and return access token
            let token_response = response
                .json::<GitHubTokenResponse>()
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
    loop {
        match get_token().await {
            Ok(token) => return Ok(token),
            Err(GitHubTokenError::NotLinked { connect_url }) => {
                // Handle connection flow
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n🔗 GitHub account not yet connected to your Stencila account.\n   Opening browser to connect your GitHub account...\n"
                );

                // Open browser
                if let Err(e) = webbrowser::open(url) {
                    eprintln!(
                        "⚠️  Failed to open browser: {}.\n   Please visit manually: {}\n",
                        e, url
                    );
                }

                // Wait for user
                eprint!("⏳ Press Enter after you've connected your account: ");
                std::io::stderr().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                eprintln!("🔄 Trying again...\n");
                // Loop will retry
            }
            Err(GitHubTokenError::RefreshFailed { connect_url }) => {
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n❌ Failed to refresh your GitHub access token.\n\n   To fix:\n   1. Visit {}\n   2. Re-connect your GitHub account\n   3. Try again\n",
                    url
                );
                bail!("GitHub token refresh failed. Please re-connect your account.");
            }
            Err(GitHubTokenError::JsonParsing(msg)) => {
                bail!("Failed to parse GitHub token response: {msg}");
            }
            Err(GitHubTokenError::Other(msg)) => {
                bail!("Failed to get GitHub access token: {msg}");
            }
        }
    }
}
