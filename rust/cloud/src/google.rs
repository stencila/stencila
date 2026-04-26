use eyre::Result;
use reqwest::Client;

use crate::tokens;

tokens::connection_token_error!(
    /// Custom error types for Google integration.
    pub enum GoogleTokenError,
    "Google"
);

/// Get a Google access token without retry
///
/// Returns a typed error that callers can handle (e.g., to open the picker on NotLinked).
/// Use `get_token_with_retry()` for interactive flows that should prompt the user.
pub async fn get_token_once() -> Result<String, GoogleTokenError> {
    tokens::get_once_default("google")
        .await
        .map_err(GoogleTokenError::from)
}

/// Get a Google access token without retry using an explicit Cloud API client.
pub async fn get_token_once_with_client(client: &Client) -> Result<String, GoogleTokenError> {
    tokens::get_once_with_client(client, "google")
        .await
        .map_err(GoogleTokenError::from)
}

/// Get Google access token with retry for connection flow
#[allow(clippy::print_stderr)]
pub async fn get_token_with_retry() -> Result<String> {
    tokens::get_with_retry("Google", "google").await
}

/// Get the Stencila Cloud Google Picker URL
///
/// Returns a URL to the Stencila Cloud Google Picker page which allows users
/// to browse their Google Drive and select a document. When a file is selected
/// via the picker, the app gains access to that specific file (within the
/// `drive.file` OAuth scope).
///
/// # Arguments
///
/// * `doc_id` - Optional Google Doc ID to pre-select or highlight in the picker
pub fn picker_url(doc_id: Option<&str>) -> String {
    const BASE: &str = "https://stencila.cloud/google/picker";
    match doc_id {
        Some(id) => {
            // Use url crate to properly encode the doc_id parameter
            let mut url = url::Url::parse(BASE).expect("valid base URL");
            url.query_pairs_mut().append_pair("doc_id", id);
            url.to_string()
        }
        None => BASE.to_string(),
    }
}
