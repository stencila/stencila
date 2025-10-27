use std::{fmt, io::Write};

use eyre::{Result, bail};
use serde::Deserialize;

use crate::{ErrorResponse, base_url, client};

/// Response from the Google token endpoint when successful
#[derive(Deserialize)]
#[allow(dead_code)]
struct GoogleTokenResponse {
    access_token: String,
    token_type: Option<String>,
    scopes: Option<String>,
}

/// Custom error types for Google integration
#[derive(Debug)]
enum GoogleTokenError {
    NotLinked { connect_url: Option<String> },
    RefreshFailed { connect_url: Option<String> },
    JsonParsing(String),
    Other(String),
}

impl fmt::Display for GoogleTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GoogleTokenError::NotLinked { connect_url } => {
                write!(
                    f,
                    "Google account not connected. Connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            GoogleTokenError::RefreshFailed { connect_url } => {
                write!(
                    f,
                    "Failed to refresh Google token. Re-connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            GoogleTokenError::JsonParsing(msg) => write!(f, "Failed to parse response: {msg}"),
            GoogleTokenError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GoogleTokenError {}

/// Get a Google access token from Stencila Cloud
///
/// This function calls the Stencila Cloud API to retrieve a Google access token
/// for the authenticated user. The user must have connected their Google account
/// to their Stencila account.
///
/// Returns a `GoogleTokenError` if the account is not connected or token refresh fails.
async fn get_token() -> Result<String, GoogleTokenError> {
    // Get authenticated client
    let client = match client().await {
        Ok(client) => client,
        Err(error) => return Err(GoogleTokenError::Other(error.to_string())),
    };

    // Call the Google token endpoint
    let url = format!("{}/connections/google/token", base_url());
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(error) => return Err(GoogleTokenError::Other(format!("Network error: {error}"))),
    };

    let status = response.status();

    // Handle different status codes
    match status.as_u16() {
        200 => {
            // Success - parse and return access token
            let token_response = response
                .json::<GoogleTokenResponse>()
                .await
                .map_err(|e| GoogleTokenError::JsonParsing(e.to_string()))?;
            Ok(token_response.access_token)
        }
        422 => {
            // Google account not connected
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| GoogleTokenError::JsonParsing(e.to_string()))?;
            Err(GoogleTokenError::NotLinked {
                connect_url: error_response.url,
            })
        }
        500 => {
            // Token refresh failed
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| GoogleTokenError::JsonParsing(e.to_string()))?;
            Err(GoogleTokenError::RefreshFailed {
                connect_url: error_response.url,
            })
        }
        _ => {
            // Other error
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("HTTP error: {status}"));
            Err(GoogleTokenError::Other(error_msg))
        }
    }
}

/// Get Google access token with retry for connection flow
#[allow(clippy::print_stderr)]
pub async fn get_token_with_retry() -> Result<String> {
    loop {
        match get_token().await {
            Ok(token) => return Ok(token),
            Err(GoogleTokenError::NotLinked { connect_url }) => {
                // Handle connection flow
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n🔗 Google account not yet connected to your Stencila account.\n   Opening browser to connect your Google account...\n"
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
            Err(GoogleTokenError::RefreshFailed { connect_url }) => {
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n❌ Failed to refresh your Google access token.\n\n   To fix:\n   1. Visit {}\n   2. Re-connect your Google account\n   3. Try again\n",
                    url
                );
                bail!("Google token refresh failed. Please re-connect your account.");
            }
            Err(GoogleTokenError::JsonParsing(msg)) => {
                bail!("Failed to parse Google token response: {msg}");
            }
            Err(GoogleTokenError::Other(msg)) => {
                bail!("Failed to get Google access token: {msg}");
            }
        }
    }
}
