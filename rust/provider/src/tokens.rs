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
use eyre::{eyre, Result};
use http_utils::{headers, post_with, serde_json::json};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize)]
struct Token {
    /// An access token for the provider
    access_token: String,

    /// The time that the access token expires
    expires_at: DateTime<Utc>,
}

/// A global store of provider tokens
static TOKENS: Lazy<RwLock<HashMap<String, Token>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Get an access token for a provider
///
/// # Arguments
///
/// - `provider`: The name of the provider (e.g. "github", "google")
///
/// Checks first for the enviroment variable `<PROVIDER>_TOKEN`.
/// Otherwise, checks for a token in `TOKENS`. If none is present, or if
/// the token is expired, then makes a request to the Stencila API for
/// a new access token.
pub async fn token_for_provider(provider: &str) -> Result<String> {
    // Is the token available in the environment?
    if let Some(access_token) = token_from_environment(provider) {
        return Ok(access_token);
    }

    // Is the token available in the store and unexpired?
    if let Some(access_token) = token_from_store(provider).await {
        return Ok(access_token);
    }

    // Get a new token from Stencila and write it to the store
    let token = token_from_stencila(provider).await?;
    TOKENS
        .write()
        .await
        .insert(provider.to_lowercase(), token.clone());
    Ok(token.access_token)
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
        if (Utc::now() + expiry_leeway) < token.expires_at {
            return Some(token.access_token.clone());
        }
    }
    None
}

/// Get an access token for a provider from the Stencila API
///
/// # Arguments
///
/// - `provider`: The name of the provider (e.g. "github", "google")
///
/// Errors if necessary enviroment variables are not present or there
/// was an error making the HTTP request.
async fn token_from_stencila(provider: &str) -> Result<Token> {
    let stencila_token = token_from_environment("stencila").ok_or_else(|| {
        eyre!("A STENCILA_TOKEN enviroment variable is required to get and refresh provider tokens")
    })?;

    let base_url =
        env::var("STENCILA_API_URL").unwrap_or_else(|_| "https://stenci.la/api/v1".to_string());

    let token: Token = post_with(
        &[&base_url, "/oauth/access_token"].concat(),
        json!({ "provider": provider.to_lowercase() }),
        &[(headers::AUTHORIZATION, stencila_token)],
    )
    .await?;
    Ok(token)
}
