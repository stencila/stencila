//! Credential persistence and loading.
//!
//! Stores and retrieves OAuth credentials in the system keyring
//! via [`stencila_secrets`]. Each provider's credentials are stored
//! as a JSON-serialized [`OAuthCredentials`] under a key like
//! `STENCILA_OAUTH_ANTHROPIC`.
//!
//! Also provides orchestration functions ([`build_oauth_token`],
//! [`load_auth_overrides`]) that assemble persisted credentials into
//! ready-to-use [`AuthCredential`] objects for the LLM client.

use std::sync::Arc;

use eyre::{Result, eyre};

use crate::{
    AuthError, AuthResult,
    credentials::{
        AuthCredential, AuthOptions, AuthOverrides, OAuthCredentials, OAuthToken, OnRefreshFn,
        RefreshFn,
    },
};

/// Secret name prefix for OAuth credentials stored in the keyring.
///
/// Each provider stores its credentials as JSON under a key like
/// `STENCILA_OAUTH_ANTHROPIC`, `STENCILA_OAUTH_GEMINI`, etc.
const OAUTH_SECRET_PREFIX: &str = "STENCILA_OAUTH";

/// Keyring secret name for a provider's OAuth credentials.
#[must_use]
pub fn secret_name(provider: &str) -> String {
    format!("{OAUTH_SECRET_PREFIX}_{}", provider.to_uppercase())
}

/// Load persisted OAuth credentials for a provider.
///
/// Returns `None` if no credentials are stored.
///
/// # Errors
///
/// Returns an error if stored credentials cannot be deserialized.
pub fn load_credentials(provider: &str) -> Result<Option<OAuthCredentials>> {
    let name = secret_name(provider);
    match stencila_secrets::get_optional(&name) {
        Ok(Some(json)) => {
            let creds: OAuthCredentials = serde_json::from_str(&json).map_err(|e| {
                eyre!("failed to parse stored OAuth credentials for {provider}: {e}")
            })?;
            Ok(Some(creds))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::warn!("Failed to read keyring for {provider} OAuth credentials: {e}");
            Ok(None)
        }
    }
}

/// Save OAuth credentials for a provider to the keyring.
///
/// # Errors
///
/// Returns an error if serialization or keyring storage fails.
pub fn save_credentials(provider: &str, credentials: &OAuthCredentials) -> Result<()> {
    let name = secret_name(provider);
    let json = serde_json::to_string(credentials)
        .map_err(|e| eyre!("failed to serialize OAuth credentials for {provider}: {e}"))?;
    stencila_secrets::set(&name, &json)
        .map_err(|e| eyre!("failed to save OAuth credentials for {provider}: {e}"))
}

/// Delete persisted OAuth credentials for a provider.
///
/// # Errors
///
/// Returns an error if keyring deletion fails.
pub fn delete_credentials(provider: &str) -> Result<()> {
    let name = secret_name(provider);
    stencila_secrets::delete(&name)
        .map_err(|e| eyre!("failed to delete OAuth credentials for {provider}: {e}"))
}

/// Create an [`OnRefreshFn`] that persists credentials to the keyring.
///
/// This is typically passed to [`crate::credentials::OAuthToken::new`]
/// so that refreshed tokens are automatically saved.
#[must_use]
pub fn on_refresh_persist(provider: &str) -> OnRefreshFn {
    let provider = provider.to_string();
    Arc::new(move |creds: &OAuthCredentials| {
        let provider = provider.clone();
        let creds = creds.clone();
        Box::pin(async move {
            if let Err(e) = save_credentials(&provider, &creds) {
                tracing::warn!("Failed to persist refreshed OAuth credentials for {provider}: {e}");
            }
        })
    })
}

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
#[allow(unused_mut)]
#[must_use]
pub fn load_auth_overrides() -> AuthOptions {
    let mut overrides = AuthOverrides::new();
    let mut openai_account_id: Option<String> = None;

    // Anthropic
    #[cfg(feature = "anthropic")]
    if let Ok(Some(creds)) = load_credentials("anthropic") {
        overrides.insert(
            "anthropic".to_string(),
            build_oauth_token(creds, crate::anthropic::refresh_fn(), None),
        );
    }

    // OpenAI — uses provider-specific load to get OpenAICredentials (includes account_id)
    #[cfg(feature = "openai")]
    if let Ok(Some(creds)) = crate::openai::load_credentials() {
        openai_account_id = Some(creds.account_id.clone());
        overrides.insert(
            "openai".to_string(),
            build_oauth_token(creds.oauth, crate::openai::refresh_fn(), None),
        );
    }

    // Gemini — needs project_id for refresh and get_api_key
    #[cfg(feature = "gemini")]
    if let Ok(Some(creds)) = crate::gemini::load_credentials() {
        let project_id = creds.project_id.clone();
        overrides.insert(
            "gemini".to_string(),
            Arc::new(OAuthToken::new(
                creds.oauth,
                crate::gemini::refresh_fn(&project_id),
                Some(crate::gemini::get_api_key_fn(&project_id)),
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

/// Convert an `eyre::Report` to an `AuthError::Authentication`.
pub(crate) fn to_auth_error(e: &eyre::Report) -> AuthError {
    AuthError::Authentication(e.to_string())
}

/// Convert an `eyre::Result` to an `AuthResult`.
pub(crate) fn to_auth_result<T>(result: Result<T>) -> AuthResult<T> {
    result.map_err(|e| to_auth_error(&e))
}
