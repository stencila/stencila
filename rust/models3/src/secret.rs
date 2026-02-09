//! Secret retrieval helpers with optional keyring support.
//!
//! When the `secrets` feature is enabled, [`get_secret`] checks environment
//! variables first and falls back to the system keyring via
//! `stencila_secrets::env_or_get`. Without the feature, only environment
//! variables are consulted.

/// Retrieve a secret value by name.
///
/// With the `secrets` feature enabled this tries `std::env::var` first,
/// then the OS keyring. Without the feature it only checks env vars.
#[cfg(feature = "secrets")]
#[must_use]
pub fn get_secret(name: &str) -> Option<String> {
    stencila_secrets::env_or_get(name).ok()
}

/// Retrieve a secret value by name (env-var only, no keyring).
#[cfg(not(feature = "secrets"))]
#[must_use]
pub fn get_secret(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Human-readable description of where secrets are read from.
///
/// Used in error messages so users know where to set their API keys.
#[cfg(feature = "secrets")]
#[must_use]
pub fn secret_source_description() -> &'static str {
    "environment variables or system keyring"
}

/// Human-readable description of where secrets are read from.
#[cfg(not(feature = "secrets"))]
#[must_use]
pub fn secret_source_description() -> &'static str {
    "environment variables"
}
