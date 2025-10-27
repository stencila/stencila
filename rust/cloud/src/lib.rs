use std::{env, sync::OnceLock, time::Duration};

use cached::proc_macro::cached;
use eyre::{Result, bail, eyre};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use strum::Display;

use stencila_version::STENCILA_USER_AGENT;

mod google;
mod microsoft;

/// The base URL for the Stencila Cloud API
///
/// Can be overridden by setting the STENCILA_API_URL environment variable.
const BASE_URL: &str = "https://api.stencila.cloud/v1";

/// Get the base URL for the Stencila Cloud API
pub fn base_url() -> String {
    env::var("STENCILA_API_URL").unwrap_or_else(|_| BASE_URL.to_string())
}

/// The name of the env var or secret for the Stencila API token
const API_TOKEN_NAME: &str = "STENCILA_API_TOKEN";

/// The API token value.
///
/// Stored to on first successful get to avoid repeated access
/// to secrets (which is relatively slow). Note that this means
/// that if the token is changed in the secrets store that the
/// process will need to be restarted for changes to take effect.
static API_TOKEN: OnceLock<String> = OnceLock::new();

/// Get the API token for the Stencila Cloud API
///
/// This function is cached (with short TTL) to avoid repeated attempts to get
/// the secret if not set. Otherwise ,this function would be called for each
/// model in the list of models to calculate the `availability` method.
#[cached(time = 15, name = "API_TOKEN_GET")]
#[tracing::instrument]
pub fn api_token() -> Option<String> {
    API_TOKEN.get().cloned().or_else(|| {
        stencila_secrets::env_or_get(API_TOKEN_NAME)
            .ok()
            .inspect(|token| {
                // If we successfully retrieved the token, store it for future use
                API_TOKEN.set(token.clone()).ok();
            })
    })
}

/// Sign in to Stencila Cloud
///
/// Sets the API token on the keyring;
pub fn signin(token: &str) -> Result<Status> {
    stencila_secrets::set(API_TOKEN_NAME, token)?;
    API_TOKEN.set(token.into()).map_err(|error| eyre!(error))?;

    Ok(status())
}

/// Sign out from Stencila Cloud
///
/// Removes the API token from the keyring. Returns the status BEFORE removal so
/// the user can be provided with appropriate messaging.
pub fn signout() -> Result<Status> {
    let status = status();
    if matches!(status.token_source, Some(TokenSource::Keyring)) {
        stencila_secrets::delete(API_TOKEN_NAME)?
    }
    Ok(status)
}

/// Get the Stencila Cloud authentication status
pub fn status() -> Status {
    let token = env::var(API_TOKEN_NAME).ok().map(stencila_secrets::redact);
    if token.is_some() {
        return Status {
            token,
            token_source: Some(TokenSource::EnvVar),
        };
    }

    let token = stencila_secrets::get(API_TOKEN_NAME)
        .ok()
        .map(stencila_secrets::redact);
    if token.is_some() {
        return Status {
            token,
            token_source: Some(TokenSource::Keyring),
        };
    }

    Status::default()
}

#[derive(Default)]
pub struct Status {
    /// The current Stencila Cloud API token (partially redacted)
    pub token: Option<String>,

    /// The source of the API token
    pub token_source: Option<TokenSource>,
}

/// The source of the current API token
#[derive(Display)]
pub enum TokenSource {
    #[strum(serialize = "keyring")]
    Keyring,
    #[strum(serialize = "environment variable")]
    EnvVar,
}

/// A request to swap a one-time code for an API token
#[derive(Serialize)]
pub struct OtcRequest {
    pub otc: String,
}

/// A response to an [`OtcRequest`]
#[derive(Deserialize)]
pub struct OtcResponse {
    pub token: String,

    #[serde(rename = "userId")]
    pub user_id: Option<String>,
}

/// An error response from Stencila Cloud
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ErrorResponse {
    pub status: u16,
    pub error: String,
    pub url: Option<String>,
}

/// Process an HTTP response from Stencila Cloud API and return parsed JSON
///
/// This function handles error responses by extracting meaningful error messages
/// and returns the parsed response body for successful requests.
pub async fn process_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    if !response.status().is_success() {
        let status = response.status();
        let message = match response.json::<ErrorResponse>().await {
            Ok(error_resp) => {
                if !error_resp.error.is_empty() {
                    error_resp.error
                } else {
                    format!("HTTP error status: {status}")
                }
            }
            Err(_) => format!("HTTP error status: {status}"),
        };
        bail!("API request failed: {message}");
    }

    response
        .json::<T>()
        .await
        .map_err(|error| eyre!("Failed to parse response: {error}"))
}

/// Get an authenticated client for the Stencila Cloud API
pub async fn client() -> Result<Client> {
    let Some(token) = api_token() else {
        bail!("Please `stencila signin` first and try again.")
    };

    let client = Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .default_headers(HeaderMap::from_iter([(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))?,
        )]))
        .build()?;

    Ok(client)
}

/// Get an access token for a remote service with interactive retry flow
///
/// This function will attempt to get a token from Stencila Cloud and handle
/// authentication failures by prompting the user to connect their account.
///
/// Currently supported services:
/// - "google": Google Drive / Google Docs
/// - "microsoft": Microsoft 365 / OneDrive
pub async fn get_token(service: &str) -> Result<String> {
    match service {
        "google" => google::get_token_with_retry().await,
        "microsoft" => microsoft::get_token_with_retry().await,
        _ => bail!("Unsupported service: {service}"),
    }
}
