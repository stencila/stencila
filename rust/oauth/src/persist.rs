//! Credential persistence helpers.
//!
//! Stores and retrieves OAuth credentials in the system keyring
//! via [`stencila_secrets`]. Each provider's credentials are stored
//! as a JSON-serialized [`OAuthCredentials`] under a key like
//! `STENCILA_OAUTH_ANTHROPIC`.

use std::sync::Arc;

use eyre::{Result, eyre};
use stencila_models3::auth::{OAuthCredentials, OnRefreshFn};
use stencila_models3::error::SdkResult;

use crate::OAUTH_SECRET_PREFIX;

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
/// This is typically passed to [`stencila_models3::auth::OAuthToken::new`]
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

/// Convert an `eyre::Report` to an `SdkError::Authentication`.
pub(crate) fn to_auth_error(e: &eyre::Report) -> stencila_models3::error::SdkError {
    stencila_models3::error::SdkError::Authentication {
        message: e.to_string(),
        details: stencila_models3::error::ProviderDetails::default(),
    }
}

/// Convert an `eyre::Report` to an `SdkResult`.
pub(crate) fn to_sdk_result<T>(result: Result<T>) -> SdkResult<T> {
    result.map_err(|e| to_auth_error(&e))
}
