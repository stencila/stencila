use std::{env, sync::OnceLock};

use cached::proc_macro::cached;
use eyre::{Context, Result, bail, eyre};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use strum::Display;

use stencila_version::STENCILA_USER_AGENT;

pub mod assets;
pub mod db;
pub mod email;
mod github;
mod google;
pub mod links;
mod microsoft;
pub mod mirror;
pub mod outputs;
pub mod sites;
mod tokens;
mod watch;
pub mod workspace;

pub use github::get_repo_token;
pub use github::{
    get_repo_token_with_client, get_token_once_with_client as github_get_token_once_with_client,
};
pub use google::{
    GoogleTokenError, get_token_once as google_get_token_once,
    get_token_once_with_client as google_get_token_once_with_client,
    picker_url as google_picker_url,
};
pub use microsoft::{
    MicrosoftTokenError, get_token_once as microsoft_get_token_once,
    get_token_once_with_client as microsoft_get_token_once_with_client,
    picker_url as microsoft_picker_url,
};
pub use watch::*;
pub use workspace::{
    WorkspaceResponse, create_or_get_workspace, create_or_get_workspace_with_client,
    ensure_workspace, ensure_workspace_with_client, get_workspace, get_workspace_with_client,
    list_workspaces, list_workspaces_with_client,
};

/// The base URL for the Stencila Cloud API
///
/// Can be overridden by setting the STENCILA_API_URL environment variable.
const BASE_URL: &str = "https://api.stencila.cloud/v1";

/// Get the base URL for the Stencila Cloud API
pub fn base_url() -> String {
    env::var("STENCILA_API_URL").unwrap_or_else(|_| BASE_URL.to_string())
}

/// The name of the env var or secret for the Stencila API key.
const API_KEY_NAME: &str = "STENCILA_API_KEY";

/// The previous name of the env var or secret for the Stencila API key.
const LEGACY_API_TOKEN_NAME: &str = "STENCILA_API_TOKEN";

/// The API key value.
///
/// Stored to on first successful get to avoid repeated access
/// to secrets (which is relatively slow). Note that this means
/// that if the key is changed in the secrets store that the
/// process will need to be restarted for changes to take effect.
static API_KEY: OnceLock<String> = OnceLock::new();

/// Get the API key for the Stencila Cloud API
///
/// This function is cached (with short TTL) to avoid repeated attempts to get
/// the secret if not set. Otherwise, this function would be called for each
/// model in the list of models to calculate the `availability` method.
#[cached(time = 15, name = "API_KEY_GET")]
#[tracing::instrument]
pub fn api_key() -> Option<String> {
    API_KEY.get().cloned().or_else(|| {
        get_api_key_value().ok().inspect(|key| {
            // If we successfully retrieved the key, store it for future use
            API_KEY.set(key.clone()).ok();
        })
    })
}

/// Get the API key for the Stencila Cloud API.
///
/// Retained for callers that have not yet been migrated to [`api_key`].
pub fn api_token() -> Option<String> {
    api_key()
}

fn get_api_key_value() -> Result<String> {
    if let Ok(key) = env::var(API_KEY_NAME) {
        return Ok(key);
    }
    if let Ok(key) = env::var(LEGACY_API_TOKEN_NAME) {
        return Ok(key);
    }

    stencila_secrets::get(API_KEY_NAME)
        .or_else(|_| stencila_secrets::get(LEGACY_API_TOKEN_NAME))
        .wrap_err_with(|| {
            format!(
                "Secret {API_KEY_NAME} is not available as an environment variable or on the keyring"
            )
        })
}

/// Sign in to Stencila Cloud
///
/// Sets the API key on the keyring.
pub fn signin(key: &str) -> Result<Status> {
    stencila_secrets::set(API_KEY_NAME, key)?;
    API_KEY.set(key.into()).ok();

    Ok(status())
}

/// Sign out from Stencila Cloud
///
/// Removes the API key from the keyring. Returns the status BEFORE removal so
/// the user can be provided with appropriate messaging.
pub fn signout() -> Result<Status> {
    let status = status();
    if matches!(status.key_source, Some(KeySource::Keyring))
        && let Some(key_name) = status.key_name
    {
        stencila_secrets::delete(key_name)?
    }
    Ok(status)
}

/// Get the Stencila Cloud authentication status
pub fn status() -> Status {
    let key = env::var(API_KEY_NAME).ok().map(stencila_secrets::redact);
    if key.is_some() {
        return Status {
            key,
            key_source: Some(KeySource::EnvVar),
            key_name: Some(API_KEY_NAME),
        };
    }

    let key = env::var(LEGACY_API_TOKEN_NAME)
        .ok()
        .map(stencila_secrets::redact);
    if key.is_some() {
        return Status {
            key,
            key_source: Some(KeySource::EnvVar),
            key_name: Some(LEGACY_API_TOKEN_NAME),
        };
    }

    let key = stencila_secrets::get(API_KEY_NAME)
        .ok()
        .map(stencila_secrets::redact);
    if key.is_some() {
        return Status {
            key,
            key_source: Some(KeySource::Keyring),
            key_name: Some(API_KEY_NAME),
        };
    }

    let key = stencila_secrets::get(LEGACY_API_TOKEN_NAME)
        .ok()
        .map(stencila_secrets::redact);
    if key.is_some() {
        return Status {
            key,
            key_source: Some(KeySource::Keyring),
            key_name: Some(LEGACY_API_TOKEN_NAME),
        };
    }

    Status::default()
}

#[derive(Default)]
pub struct Status {
    /// The current Stencila Cloud API key (partially redacted)
    pub key: Option<String>,

    /// The source of the API key
    pub key_source: Option<KeySource>,

    /// The name of the env var or secret containing the API key
    pub key_name: Option<&'static str>,
}

/// The source of the current API key
#[derive(Display)]
pub enum KeySource {
    #[strum(serialize = "keyring")]
    Keyring,
    #[strum(serialize = "environment variable")]
    EnvVar,
}

/// A request to swap a one-time code for an API key
#[derive(Serialize)]
pub struct OtcRequest {
    pub otc: String,
}

/// A response to an [`OtcRequest`]
#[derive(Deserialize)]
pub struct OtcResponse {
    #[serde(alias = "token")]
    pub key: String,

    #[serde(rename = "userId")]
    pub user_id: Option<String>,
}

/// An error response from Stencila Cloud
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ErrorResponse {
    pub status: u16,
    pub error: String,
    pub advice: Option<String>,
    pub url: Option<String>,
}

impl ErrorResponse {
    fn message(self) -> String {
        let mut message = self.error;
        if let Some(advice) = self.advice {
            message.push(' ');
            message.push_str(&advice);
        }
        if let Some(url) = self.url {
            message.push(' ');
            message.push_str(&url);
        }
        message
    }
}

/// Check an HTTP response from Stencila Cloud API for errors
///
/// This function handles error responses by extracting meaningful error messages.
/// Use this for responses with no body (e.g., DELETE operations returning 204 No Content).
pub async fn check_response(response: reqwest::Response) -> Result<()> {
    if !response.status().is_success() {
        let status = response.status();
        let message = match response.json::<ErrorResponse>().await {
            Ok(error_response) => {
                if !error_response.error.is_empty() {
                    error_response.message()
                } else {
                    format!("HTTP error status: {status}")
                }
            }
            Err(_) => format!("HTTP error status: {status}"),
        };
        bail!("{message}");
    }

    Ok(())
}

/// Process an HTTP response from Stencila Cloud API and return parsed JSON
///
/// This function handles error responses by extracting meaningful error messages
/// and returns the parsed response body for successful requests.
#[tracing::instrument]
pub async fn process_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    if !response.status().is_success() {
        let status = response.status();
        let message = match response.json::<ErrorResponse>().await {
            Ok(error_response) => {
                if !error_response.error.is_empty() {
                    error_response.message()
                } else {
                    format!("HTTP error status: {status}")
                }
            }
            Err(_) => format!("HTTP error status: {status}"),
        };
        bail!("{message}");
    }

    // Get response text first for better error messages
    let text = response.text().await?;

    serde_json::from_str(&text).map_err(|error| {
        // Include a snippet of the response for debugging
        let snippet = if text.len() > 200 {
            format!("{}...", &text[..200])
        } else {
            text.clone()
        };
        eyre!("Failed to parse response: {error}\nResponse: {snippet}")
    })
}

/// Build an authenticated client for the Stencila Cloud API with an explicit API key.
pub fn client_with_api_key(key: &str) -> Result<Client> {
    let client = Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .default_headers(HeaderMap::from_iter([(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {key}"))?,
        )]))
        .build()?;

    Ok(client)
}

/// Build an authenticated client for the Stencila Cloud API
pub async fn client() -> Result<Client> {
    let Some(key) = api_key() else {
        bail!("Please *stencila signin* first and try again.")
    };

    client_with_api_key(&key)
}

/// Build an unauthenticated client for the Stencila Cloud API
fn public_client() -> Result<Client> {
    Ok(Client::builder().user_agent(STENCILA_USER_AGENT).build()?)
}

/// Get an access token for a remote service with interactive retry flow
///
/// This function will attempt to get a token from Stencila Cloud and handle
/// authentication failures by prompting the user to connect their account.
///
/// Currently supported services:
/// - "github": GitHub
/// - "google": Google Drive / Google Docs
/// - "microsoft": Microsoft 365 / OneDrive
pub async fn get_token(service: &str) -> Result<String> {
    match service {
        "github" => github::get_token_with_retry().await,
        "google" => google::get_token_with_retry().await,
        "microsoft" => microsoft::get_token_with_retry().await,
        _ => bail!("Unsupported service: {service}"),
    }
}

/// A log entry from a session
#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Response from fetching logs, including completion status
#[derive(Debug)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
    pub is_complete: bool,
}

/// Get logs for a session from Stencila Cloud
///
/// # Arguments
///
/// * `session_id` - The ID of the session to retrieve logs for
///
/// # Returns
///
/// A `LogsResponse` containing log entries and whether logs are complete
pub async fn get_logs(session_id: &str) -> Result<LogsResponse> {
    let client = client().await?;
    let url = format!("{}/sessions/{}/logs", base_url(), session_id);
    let response = client.get(&url).send().await?;

    // Check for X-Logs-Complete header before consuming response
    let is_complete = response
        .headers()
        .get("X-Logs-Complete")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let logs = process_response(response).await?;

    Ok(LogsResponse { logs, is_complete })
}
