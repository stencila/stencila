use std::{env, sync::OnceLock};

use cached::proc_macro::cached;

use common::{serde::Deserialize, tracing};

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
        secrets::env_or_get(API_TOKEN_NAME).ok().inspect(|token| {
            // If we successfully retrieved the token, store it for future use
            API_TOKEN.set(token.clone()).ok();
        })
    })
}

/// An error response from Stencila Cloud
#[derive(Default, Deserialize)]
#[serde(default, crate = "common::serde")]
pub struct ErrorResponse {
    pub status: u16,
    pub error: String,
}
