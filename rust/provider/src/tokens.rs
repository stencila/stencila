//! Utilities for obtaining and refreshing access tokens for providers
//!
//! Some providers require authentication tokens to be able to access
//! the APIs of the provider on the behalf of a user (e.g Google for everything,
//! Github for private repos).
//!
//! Some access tokens expire after a certain period and need to be refreshed.
//! In OAuth2 a "refresh token" is used, along with a client secret and client id
//! to generate a new access token for a user.
//!
//! This module provides a unified way of obtaining provider access tokens from the
//! environment (e.g. `GITHUB_TOKEN`) falling back to fetching an fresh token from
//! the Stencila API.

use std::{collections::HashMap, env};

use chrono::{DateTime, Duration, Utc};
use eyre::{bail, Result};
use http_utils::{get_response, headers, serde_json};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Token {
    /// An access token for the provider
    access_token: String,

    /// The time that the access token expires
    expires_at: Option<DateTime<Utc>>,
}

/// A global store of provider tokens
static TOKENS: Lazy<RwLock<HashMap<String, Token>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Get an access token for a provider
///
/// # Arguments
///
/// - `provider`: The name of the provider (e.g. "github", "google")
///
/// Checks first for the environment variable `<PROVIDER>_TOKEN`.
/// If none is available, checks for a token in `TOKENS` (we don't do this first
/// because we want to allow changing of the env var if necessary).
/// If none is present in `TOKENS`, or if the token is expired, then makes a request
/// to the Stencila API for a fresh access token.
/// If the user has not authenticated with Stencila using the provider will return `Ok(None)`.
///
/// Will error if there is a problem connecting to the Stencila API.
pub async fn token_for_provider(provider: &str) -> Result<Option<String>> {
    // Is the token available in the environment?
    if let Some(access_token) = token_from_environment(provider) {
        return Ok(Some(access_token));
    }

    // Is the token available in the store and unexpired?
    if let Some(access_token) = token_from_store(provider).await {
        return Ok(Some(access_token));
    }

    // Get a new token from Stencila and write it to the store
    if let Some(token) = token_from_stencila(provider).await? {
        TOKENS
            .write()
            .await
            .insert(provider.to_lowercase(), token.clone());
        return Ok(Some(token.access_token));
    }

    Ok(None)
}

/// Get an access token for a provider from the enviroment
///
/// # Arguments
///
/// - `provider`: The name of the provider (e.g. "github", "google")
///
/// Returns the enviroment variable `<PROVIDER>_TOKEN` (if any)
/// where `<PROVIDER>` is the uppercase of the `provider` argument.
fn token_from_environment(provider: &str) -> Option<String> {
    let name = [provider, "_TOKEN"].concat().to_uppercase();
    env::var(name).ok()
}

/// Get an unexpired access token for the provider from `TOKENS`
///
/// # Arguments
///
/// - `provider`: The name of the provider (e.g. "github", "google")
///
/// Returns `None` if there is no token, or if an existing token has expired.
async fn token_from_store(provider: &str) -> Option<String> {
    if let Some(token) = TOKENS.read().await.get(&provider.to_lowercase()) {
        let expiry_leeway = Duration::seconds(60);
        if let Some(expires_at) = token.expires_at {
            if Utc::now() > (expires_at - expiry_leeway) {
                return None;
            }
        }
        return Some(token.access_token.clone());
    }
    None
}

/// Get an access token for a provider from the Stencila API
///
/// # Arguments
///
/// - `provider`: The name of the provider (e.g. "github", "google")
///
/// Errors if necessary environment variables are not present or there
/// was an error making the HTTP request. Returns `Ok(None)` if a token is
/// not available for the provider from Stencila i.e. that the user has not
/// authenticated with the provider.
async fn token_from_stencila(provider: &str) -> Result<Option<Token>> {
    let base_url =
        env::var("STENCILA_API_URL").unwrap_or_else(|_| "https://stenci.la/api/v1".to_string());

    let stencila_token = match token_from_environment("stencila") {
        Some(token) => token,
        None => return Ok(None),
    };

    let response = get_response(
        &[
            &base_url,
            "/oauth/token?provider=",
            &provider.to_lowercase(),
        ]
        .concat(),
        &[(headers::AUTHORIZATION, format!("Token {}", stencila_token))],
    )
    .await?;

    let payload: serde_json::Value = response.json().await?;
    if let Some(error) = payload.get("error") {
        bail!(
            "While fetching token for `{}` from Stencila: {}",
            provider,
            error
                .get("message")
                .and_then(|msg| msg.as_str())
                .unwrap_or_default()
        )
    }

    let token: Token = serde_json::from_value(payload)?;
    Ok(Some(token))
}
