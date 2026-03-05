//! Secret retrieval helpers.
//!
//! [`get_secret`] checks environment variables first and falls back to the
//! system keyring via `stencila_secrets::env_or_get`.

/// Retrieve a secret value by name.
///
/// Tries `std::env::var` first, then the OS keyring.
#[must_use]
pub fn get_secret(name: &str) -> Option<String> {
    stencila_secrets::env_or_get(name).ok()
}

/// Human-readable description of where secrets are read from.
///
/// Used in error messages so users know where to set their API keys.
#[must_use]
pub fn secret_source_description() -> &'static str {
    "environment variables or system keyring"
}
