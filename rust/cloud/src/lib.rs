use std::{env, sync::OnceLock};

use common::serde::Deserialize;

/// The base URL for the Stencila Cloud API
///
/// Can be overridden by setting the STENCILA_API_URL environment variable.
const BASE_URL: &str = "https://api.stencila.cloud/v1";

/// Get the base URL for the Stencila Cloud API
pub fn base_url() -> String {
    env::var("STENCILA_API_URL").unwrap_or_else(|_| BASE_URL.to_string())
}

/// The name of the env var or secret for the API key
const API_KEY_NAME: &str = "STENCILA_API_TOKEN";

/// The API key value.
///
/// Stored to on first successful get to avoid repeated access
/// to secrets (which is relatively slow). Note that this means
/// that if the key is changed in the secrets store that the
/// process will need to be restarted for changes to take effect.
static API_KEY: OnceLock<String> = OnceLock::new();

/// Get the API key for the Stencila Cloud API
pub fn api_key() -> Option<String> {
    API_KEY.get().cloned().or_else(|| {
        secrets::env_or_get(API_KEY_NAME).ok().map(|key| {
            // If we successfully retrieved the key, store it for future use
            API_KEY.set(key.clone()).ok();
            key
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
