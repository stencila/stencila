use eyre::Result;
use reqwest::Client;

use crate::tokens;

tokens::connection_token_error!(
    /// Custom error types for Microsoft integration.
    pub enum MicrosoftTokenError,
    "Microsoft"
);

/// Get a Microsoft access token without retry
///
/// Returns a typed error that callers can handle (e.g., to open the picker on NotLinked).
/// Use `get_token_with_retry()` for interactive flows that should prompt the user.
pub async fn get_token_once() -> Result<String, MicrosoftTokenError> {
    tokens::get_once_default("microsoft")
        .await
        .map_err(MicrosoftTokenError::from)
}

/// Get a Microsoft access token without retry using an explicit Cloud API client.
pub async fn get_token_once_with_client(client: &Client) -> Result<String, MicrosoftTokenError> {
    tokens::get_once_with_client(client, "microsoft")
        .await
        .map_err(MicrosoftTokenError::from)
}

/// Get Microsoft access token with retry for connection flow
#[allow(clippy::print_stderr)]
pub async fn get_token_with_retry() -> Result<String> {
    tokens::get_with_retry("Microsoft", "microsoft").await
}

/// Get the Stencila Cloud Microsoft Picker URL
///
/// Returns a URL to the Stencila Cloud Microsoft Picker page which allows users
/// to browse their OneDrive and select a document. When a file is selected
/// via the picker, the app gains access to that specific file.
///
/// # Arguments
///
/// * `doc_id` - Optional OneDrive item ID to pre-select or highlight in the picker
pub fn picker_url(doc_id: Option<&str>) -> String {
    const BASE: &str = "https://stencila.cloud/microsoft/picker";
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
