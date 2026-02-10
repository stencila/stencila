//! Authentication types and OAuth login flows for LLM providers.
//!
//! The [`auth`] module provides the core credential abstraction
//! ([`auth::AuthCredential`], [`auth::StaticKey`], [`auth::OAuthToken`])
//! used by LLM client crates. These types are always available, even
//! without the `login` feature.
//!
//! When the `login` feature is enabled, this crate also provides
//! browser-based OAuth login flows for specific providers:
//!
//! - **Anthropic** — Authorization Code + PKCE with manual code paste.
//! - **Gemini** — Authorization Code + PKCE with local callback server.
//! - **`OpenAI` (Codex)** — Authorization Code + PKCE with local callback server.
//! - **GitHub Copilot** — Device Code Grant with poll-based verification.
//!
//! Credentials are persisted to the system keyring via [`stencila_secrets`]
//! and can be loaded into [`auth::OAuthToken`] for automatic refresh during
//! API calls.

#![allow(clippy::result_large_err)]
#![warn(clippy::pedantic)]

/// Authentication credentials (static keys and OAuth tokens).
mod auth;
pub use auth::*;

/// Anthropic OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "anthropic")]
pub mod anthropic;

/// Auto-detect Claude Code OAuth credentials.
#[cfg(feature = "login")]
pub mod claude_code;

/// Local HTTP callback server for OAuth redirects.
#[cfg(feature = "login")]
pub mod callback;

/// Auto-detect Codex CLI OAuth credentials.
#[cfg(feature = "login")]
pub mod codex_cli;

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
#[cfg(feature = "login")]
pub mod pkce;

/// Shared utilities for credential persistence and token helpers.
#[cfg(feature = "login")]
pub mod persist;

#[cfg(feature = "login")]
use std::sync::Arc;

/// Secret name prefix for OAuth credentials stored in the keyring.
///
/// Each provider stores its credentials as JSON under a key like
/// `STENCILA_OAUTH_ANTHROPIC`, `STENCILA_OAUTH_GEMINI`, etc.
#[cfg(feature = "login")]
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
#[cfg(feature = "login")]
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
#[cfg(feature = "login")]
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
