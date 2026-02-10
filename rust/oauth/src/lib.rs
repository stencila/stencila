//! Direct OAuth login flows for LLM providers.
//!
//! This crate implements browser-based OAuth flows that obtain credentials
//! for use with the [`stencila_models3`] client. Each provider module
//! handles the specific OAuth grant type required by that provider:
//!
//! - **Anthropic** — Authorization Code + PKCE with manual code paste.
//! - **Gemini** — Authorization Code + PKCE with local callback server.
//! - **OpenAI (Codex)** — Authorization Code + PKCE with local callback server.
//! - **GitHub Copilot** — Device Code Grant with poll-based verification.
//!
//! Credentials are persisted to the system keyring via [`stencila_secrets`]
//! and can be loaded into [`stencila_models3::auth::OAuthToken`] for
//! automatic refresh during API calls.

#![allow(clippy::result_large_err)]
#![warn(clippy::pedantic)]

/// Anthropic OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "anthropic")]
pub mod anthropic;

/// Local HTTP callback server for OAuth redirects.
pub mod callback;

/// Command-line interface for OAuth login commands.
#[cfg(feature = "cli")]
pub mod cli;

/// GitHub Copilot OAuth login flow (Device Code Grant).
#[cfg(feature = "copilot")]
pub mod copilot;

/// Google Gemini OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "gemini")]
pub mod gemini;

/// OpenAI OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "openai")]
pub mod openai;

/// PKCE (Proof Key for Code Exchange) challenge generation.
pub mod pkce;

/// Shared utilities for credential persistence and token helpers.
pub mod persist;

use std::sync::Arc;

use stencila_models3::auth::{
    AuthCredential, OAuthCredentials, OAuthToken, OnRefreshFn, RefreshFn,
};
use stencila_models3::client::{AuthOptions, AuthOverrides};

/// Secret name prefix for OAuth credentials stored in the keyring.
///
/// Each provider stores its credentials as JSON under a key like
/// `STENCILA_OAUTH_ANTHROPIC`, `STENCILA_OAUTH_GEMINI`, etc.
const OAUTH_SECRET_PREFIX: &str = "STENCILA_OAUTH";

/// Build an [`OAuthToken`] from persisted credentials and provider-specific callbacks.
///
/// This is the primary entry point for constructing an [`AuthCredential`]
/// from previously-stored OAuth credentials. The returned token will
/// auto-refresh when expired.
///
/// # Errors
///
/// Returns an error if the credential JSON cannot be deserialized.
pub fn build_oauth_token(
    credentials: OAuthCredentials,
    refresh_fn: RefreshFn,
    on_refresh_fn: Option<OnRefreshFn>,
) -> Arc<dyn AuthCredential> {
    Arc::new(OAuthToken::new(
        credentials,
        refresh_fn,
        None,
        on_refresh_fn,
    ))
}

/// Load all persisted OAuth credentials and build [`AuthOptions`].
///
/// Checks each provider's keyring entry. For providers with stored
/// credentials, builds an [`OAuthToken`] with the appropriate refresh
/// function and adds it to the options map.
///
/// Providers without stored credentials are silently skipped.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn load_auth_overrides() -> AuthOptions {
    let mut overrides = AuthOverrides::new();
    let mut openai_account_id: Option<String> = None;

    // Anthropic
    #[cfg(feature = "anthropic")]
    if let Ok(Some(creds)) = persist::load_credentials("anthropic") {
        overrides.insert(
            "anthropic".to_string(),
            build_oauth_token(creds, anthropic::refresh_fn(), None),
        );
    }

    // OpenAI — uses provider-specific load to get OpenAICredentials (includes account_id)
    #[cfg(feature = "openai")]
    if let Ok(Some(creds)) = openai::load_credentials() {
        openai_account_id = Some(creds.account_id.clone());
        overrides.insert(
            "openai".to_string(),
            build_oauth_token(creds.oauth, openai::refresh_fn(), None),
        );
    }

    // Gemini — needs project_id for refresh and get_api_key
    #[cfg(feature = "gemini")]
    if let Ok(Some(creds)) = gemini::load_credentials() {
        let project_id = creds.project_id.clone();
        overrides.insert(
            "gemini".to_string(),
            Arc::new(OAuthToken::new(
                creds.oauth,
                gemini::refresh_fn(&project_id),
                Some(gemini::get_api_key_fn(&project_id)),
                None,
            )),
        );
    }

    // Copilot — needs enterprise_domain for refresh; maps to openai_chat_completions
    // Note: Copilot uses the OpenAI Chat Completions API, but the adapter
    // requires additional base URL configuration that isn't handled by
    // simple auth overrides. Copilot integration is deferred.

    AuthOptions {
        overrides,
        openai_account_id,
    }
}
