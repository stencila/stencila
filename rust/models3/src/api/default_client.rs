use std::sync::{OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::client::Client;
use crate::error::SdkResult;

static DEFAULT_CLIENT: OnceLock<RwLock<Option<&'static Client>>> = OnceLock::new();

fn store() -> &'static RwLock<Option<&'static Client>> {
    DEFAULT_CLIENT.get_or_init(|| RwLock::new(None))
}

fn read_store() -> RwLockReadGuard<'static, Option<&'static Client>> {
    store().read().unwrap_or_else(PoisonError::into_inner)
}

fn write_store() -> RwLockWriteGuard<'static, Option<&'static Client>> {
    store().write().unwrap_or_else(PoisonError::into_inner)
}

/// Set the default client used by `generate()` / `stream()` when
/// no explicit `client` is passed.
///
/// Replaces any previously configured default client.
pub fn set_default_client(client: Client) {
    let leaked = Box::leak(Box::new(client));
    let mut guard = write_store();
    *guard = Some(leaked);
}

/// Get a reference to the default client.
///
/// If [`set_default_client()`] was called, returns that client.
/// Otherwise, lazily creates one from environment variables via
/// [`Client::from_env()`].
///
/// # Errors
///
/// Returns `SdkError::Configuration` if no default client is set
/// and `Client::from_env()` fails (no API keys configured, etc.).
pub fn get_default_client() -> SdkResult<&'static Client> {
    if let Some(client) = *read_store() {
        return Ok(client);
    }

    // Attempt lazy initialization from environment.
    let client = Client::from_env()?;
    let mut guard = write_store();
    if let Some(existing) = *guard {
        return Ok(existing);
    }

    let leaked = Box::leak(Box::new(client));
    *guard = Some(leaked);
    Ok(leaked)
}
