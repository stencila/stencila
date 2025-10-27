use std::{fmt, io::Write};

use eyre::{Result, bail};
use serde::Deserialize;

use crate::{ErrorResponse, base_url, client};

/// Response from the Microsoft token endpoint when successful
#[derive(Deserialize)]
#[allow(dead_code)]
struct MicrosoftTokenResponse {
    access_token: String,
    token_type: Option<String>,
    scopes: Option<String>,
}

/// Custom error types for Microsoft integration
#[derive(Debug)]
enum MicrosoftTokenError {
    NotLinked { connect_url: Option<String> },
    RefreshFailed { connect_url: Option<String> },
    JsonParsing(String),
    Other(String),
}

impl fmt::Display for MicrosoftTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MicrosoftTokenError::NotLinked { connect_url } => {
                write!(
                    f,
                    "Microsoft account not connected. Connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            MicrosoftTokenError::RefreshFailed { connect_url } => {
                write!(
                    f,
                    "Failed to refresh Microsoft token. Re-connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            MicrosoftTokenError::JsonParsing(msg) => write!(f, "Failed to parse response: {msg}"),
            MicrosoftTokenError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for MicrosoftTokenError {}

/// Get a Microsoft access token from Stencila Cloud
///
/// This function calls the Stencila Cloud API to retrieve a Microsoft access token
/// for the authenticated user. The user must have connected their Microsoft account
/// to their Stencila account.
///
/// Returns a `MicrosoftTokenError` if the account is not connected or token refresh fails.
async fn get_token() -> Result<String, MicrosoftTokenError> {
    // Get authenticated client
    let client = match client().await {
        Ok(client) => client,
        Err(error) => return Err(MicrosoftTokenError::Other(error.to_string())),
    };

    // Call the Microsoft token endpoint
    let url = format!("{}/connections/microsoft/token", base_url());
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(error) => {
            return Err(MicrosoftTokenError::Other(format!(
                "Network error: {error}"
            )));
        }
    };

    let status = response.status();

    // Handle different status codes
    match status.as_u16() {
        200 => {
            // Success - parse and return access token
            let token_response = response
                .json::<MicrosoftTokenResponse>()
                .await
                .map_err(|e| MicrosoftTokenError::JsonParsing(e.to_string()))?;
            Ok(token_response.access_token)
        }
        422 => {
            // Microsoft account not connected
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| MicrosoftTokenError::JsonParsing(e.to_string()))?;
            Err(MicrosoftTokenError::NotLinked {
                connect_url: error_response.url,
            })
        }
        500 => {
            // Token refresh failed
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| MicrosoftTokenError::JsonParsing(e.to_string()))?;
            Err(MicrosoftTokenError::RefreshFailed {
                connect_url: error_response.url,
            })
        }
        _ => {
            // Other error
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("HTTP error: {status}"));
            Err(MicrosoftTokenError::Other(error_msg))
        }
    }
}

/// Get Microsoft access token with retry for connection flow
pub async fn get_token_with_retry() -> Result<String> {
    loop {
        match get_token().await {
            Ok(token) => return Ok(token),
            Err(MicrosoftTokenError::NotLinked { connect_url }) => {
                // Handle connection flow
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n🔗 Microsoft account not yet connected to your Stencila account.\n   Opening browser to connect your Microsoft account...\n"
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
            Err(MicrosoftTokenError::RefreshFailed { connect_url }) => {
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n❌ Failed to refresh your Microsoft access token.\n\n   To fix:\n   1. Visit {}\n   2. Re-connect your Microsoft account\n   3. Try again\n",
                    url
                );
                bail!("Microsoft token refresh failed. Please re-connect your account.");
            }
            Err(MicrosoftTokenError::JsonParsing(msg)) => {
                bail!("Failed to parse Microsoft token response: {msg}");
            }
            Err(MicrosoftTokenError::Other(msg)) => {
                bail!("Failed to get Microsoft access token: {msg}");
            }
        }
    }
}
