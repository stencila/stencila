use std::fmt;

use eyre::Result;
use serde::Deserialize;

use crate::{ErrorResponse, base_url, client};

/// Response from the Google token endpoint when successful
#[derive(Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub scopes: Option<String>,
}

/// Custom error types for Google integration
#[derive(Debug)]
pub enum GoogleTokenError {
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
pub async fn google_token() -> Result<String, GoogleTokenError> {
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
