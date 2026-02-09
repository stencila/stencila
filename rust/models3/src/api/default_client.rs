use std::sync::OnceLock;

use crate::client::Client;
use crate::error::{SdkError, SdkResult};

static DEFAULT_CLIENT: OnceLock<Client> = OnceLock::new();

/// Set the default client used by `generate()` / `stream()` when
/// no explicit `client` is passed.
///
/// Can only be called once. Subsequent calls are silently ignored.
pub fn set_default_client(client: Client) {
    let _ = DEFAULT_CLIENT.set(client);
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
    if let Some(c) = DEFAULT_CLIENT.get() {
        return Ok(c);
    }

    // Attempt lazy initialization from environment
    let client = Client::from_env()?;
    // Race-safe: OnceLock::set is atomic; if another thread won, use theirs
    let _ = DEFAULT_CLIENT.set(client);
    DEFAULT_CLIENT.get().ok_or_else(|| SdkError::Configuration {
        message: "failed to initialize default client".into(),
    })
}
