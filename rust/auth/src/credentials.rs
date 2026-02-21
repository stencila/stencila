//! Authentication credentials for LLM provider API calls.
//!
//! Provides a trait-based abstraction over different authentication methods:
//!
//! - [`StaticKey`] — wraps a plain API key string (the common case).
//! - [`OAuthToken`] — manages expiring OAuth credentials with automatic refresh.
//!
//! Provider adapters hold an `Arc<dyn AuthCredential>` and call
//! [`AuthCredential::get_token`] on each request, so tokens can be
//! refreshed transparently without rebuilding the adapter or client.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// BoxFuture (same definition as in models3's provider.rs)
// ---------------------------------------------------------------------------

/// A boxed future that is Send.
pub type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Error type for authentication operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    /// An authentication operation failed (e.g. token refresh).
    #[error("authentication error: {0}")]
    Authentication(String),
    /// A configuration issue (e.g. invalid header value).
    #[error("configuration error: {0}")]
    Configuration(String),
}

/// Convenience alias for `Result<T, AuthError>`.
pub type AuthResult<T> = Result<T, AuthError>;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A credential that can produce a valid API key or access token.
///
/// Implementations that manage expiring tokens (e.g. [`OAuthToken`]) should
/// handle refresh transparently before returning.
pub trait AuthCredential: Send + Sync {
    /// Return a valid API key or access token for making LLM API calls.
    fn get_token(&self) -> BoxFuture<'_, AuthResult<String>>;
}

impl std::fmt::Debug for dyn AuthCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("dyn AuthCredential")
    }
}

// ---------------------------------------------------------------------------
// StaticKey
// ---------------------------------------------------------------------------

/// A non-expiring API key credential.
///
/// Wraps a plain `String` and returns it directly from [`get_token`](AuthCredential::get_token).
/// All existing `new(api_key)` / `from_env()` code paths use this.
#[derive(Clone)]
pub struct StaticKey(String);

impl StaticKey {
    /// Create a new static key credential.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl std::fmt::Debug for StaticKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticKey").field(&"***").finish()
    }
}

impl AuthCredential for StaticKey {
    fn get_token(&self) -> BoxFuture<'_, AuthResult<String>> {
        let token = self.0.clone();
        Box::pin(async move { Ok(token) })
    }
}

// ---------------------------------------------------------------------------
// OAuthCredentials
// ---------------------------------------------------------------------------

/// Persistent OAuth credentials.
///
/// Serializable so callers can persist to keyring/disk across sessions.
/// Follows the pi-mono `OAuthCredentials` pattern: refresh tokens may
/// rotate on each refresh, so the entire struct is returned from refresh
/// callbacks.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthCredentials {
    /// The refresh token used to obtain new access tokens.
    pub refresh_token: String,
    /// The current access token.
    pub access_token: String,
    /// When the access token expires (Unix timestamp in milliseconds).
    /// `None` means the token does not expire.
    pub expires_at: Option<u64>,
}

// ---------------------------------------------------------------------------
// Callback type aliases
// ---------------------------------------------------------------------------

/// Callback to refresh expired credentials.
///
/// Takes the current credentials and returns new credentials (the refresh
/// token may have rotated).
pub type RefreshFn =
    Arc<dyn Fn(OAuthCredentials) -> BoxFuture<'static, AuthResult<OAuthCredentials>> + Send + Sync>;

/// Callback to convert credentials to the value the LLM API needs.
///
/// Default behavior (when not provided): returns `credentials.access_token`.
/// Override for providers like GitHub Copilot that need an extra exchange step.
pub type GetApiKeyFn =
    Arc<dyn Fn(&OAuthCredentials) -> BoxFuture<'_, AuthResult<String>> + Send + Sync>;

/// Callback invoked after credentials are refreshed.
///
/// The caller uses this to persist updated credentials to keyring/disk
/// so they survive process restarts.
pub type OnRefreshFn = Arc<dyn Fn(&OAuthCredentials) -> BoxFuture<'_, ()> + Send + Sync>;

// ---------------------------------------------------------------------------
// OAuthToken
// ---------------------------------------------------------------------------

/// Buffer before actual expiry at which a refresh is triggered (5 minutes).
const EXPIRY_BUFFER_MS: u64 = 5 * 60 * 1000;

/// An OAuth credential that auto-refreshes expired access tokens.
///
/// Holds cached credentials behind an `RwLock` and uses singleflight
/// semantics: the first caller that discovers an expired token performs
/// the refresh while concurrent callers wait.
pub struct OAuthToken {
    state: RwLock<OAuthState>,
    refresh_fn: RefreshFn,
    get_api_key_fn: Option<GetApiKeyFn>,
    on_refresh_fn: Option<OnRefreshFn>,
}

struct OAuthState {
    credentials: OAuthCredentials,
    refreshing: bool,
}

impl std::fmt::Debug for OAuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthToken").finish_non_exhaustive()
    }
}

impl OAuthToken {
    /// Create a new OAuth token manager.
    ///
    /// - `credentials` — initial OAuth credentials (from a prior login or persisted state).
    /// - `refresh_fn` — called when the access token has expired or is about to expire.
    /// - `get_api_key_fn` — optional; converts credentials to the API key string.
    ///   Defaults to returning `credentials.access_token`.
    /// - `on_refresh_fn` — optional; called after a successful refresh so the caller
    ///   can persist the new credentials.
    pub fn new(
        credentials: OAuthCredentials,
        refresh_fn: RefreshFn,
        get_api_key_fn: Option<GetApiKeyFn>,
        on_refresh_fn: Option<OnRefreshFn>,
    ) -> Self {
        Self {
            state: RwLock::new(OAuthState {
                credentials,
                refreshing: false,
            }),
            refresh_fn,
            get_api_key_fn,
            on_refresh_fn,
        }
    }

    /// Check whether the credentials are expired or about to expire.
    fn is_expired(credentials: &OAuthCredentials) -> bool {
        let Some(expires_at) = credentials.expires_at else {
            return false;
        };
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|d| u64::try_from(d.as_millis()).ok())
            .unwrap_or(0);
        now_ms + EXPIRY_BUFFER_MS >= expires_at
    }
}

impl AuthCredential for OAuthToken {
    fn get_token(&self) -> BoxFuture<'_, AuthResult<String>> {
        Box::pin(async move {
            loop {
                // Fast path: read lock — token is valid and no refresh in progress.
                {
                    let state = self.state.read().await;
                    if !state.refreshing && !Self::is_expired(&state.credentials) {
                        let creds = &state.credentials;
                        return if let Some(ref get_key) = self.get_api_key_fn {
                            get_key(creds).await
                        } else {
                            Ok(creds.access_token.clone())
                        };
                    }
                    if state.refreshing {
                        // Another task is refreshing — drop the lock and yield.
                        drop(state);
                        tokio::task::yield_now().await;
                        continue;
                    }
                }

                // Slow path: acquire write lock and perform refresh.
                {
                    let mut state = self.state.write().await;

                    // Double-check: another task may have refreshed while we waited.
                    if !Self::is_expired(&state.credentials) {
                        let creds = &state.credentials;
                        return if let Some(ref get_key) = self.get_api_key_fn {
                            get_key(creds).await
                        } else {
                            Ok(creds.access_token.clone())
                        };
                    }
                    if state.refreshing {
                        drop(state);
                        tokio::task::yield_now().await;
                        continue;
                    }

                    // Mark as refreshing.
                    state.refreshing = true;
                    let old_creds = state.credentials.clone();
                    drop(state);

                    // Perform the refresh outside the lock.
                    let result = (self.refresh_fn)(old_creds).await;

                    let mut state = self.state.write().await;
                    state.refreshing = false;

                    match result {
                        Ok(new_creds) => {
                            state.credentials = new_creds;
                            if let Some(ref on_refresh) = self.on_refresh_fn {
                                on_refresh(&state.credentials).await;
                            }
                            let creds = &state.credentials;
                            return if let Some(ref get_key) = self.get_api_key_fn {
                                get_key(creds).await
                            } else {
                                Ok(creds.access_token.clone())
                            };
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// AuthOverrides and AuthOptions
// ---------------------------------------------------------------------------

/// Per-provider authentication overrides.
///
/// Maps provider names (e.g. `"openai"`, `"anthropic"`) to credential
/// objects. When passed to a client constructor, providers with an override
/// use the supplied credential instead of reading API keys from environment
/// variables or the system keyring.
pub type AuthOverrides = HashMap<String, Arc<dyn AuthCredential>>;

/// Authentication configuration for client construction.
///
/// Wraps [`AuthOverrides`] together with provider-specific metadata
/// that cannot be expressed through the [`AuthCredential`] trait alone
/// (e.g. the `OpenAI` `ChatGPT-Account-Id` header required for OAuth tokens).
#[derive(Default)]
pub struct AuthOptions {
    /// Per-provider credential overrides.
    pub overrides: AuthOverrides,
    /// `OpenAI` `ChatGPT` account ID extracted from the JWT during OAuth login.
    ///
    /// Sent as the `ChatGPT-Account-Id` header with every `OpenAI` API request
    /// when authenticating via OAuth (Codex flow).
    pub openai_account_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `HeaderMap` containing a single `Authorization: Bearer {token}` entry.
///
/// Used by provider adapters that inject auth per-request.
///
/// # Errors
///
/// Returns `AuthError::Configuration` if the token contains characters
/// that are invalid in an HTTP header value.
pub fn bearer_header(token: &str) -> AuthResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value: HeaderValue = format!("Bearer {token}")
        .parse()
        .map_err(|e| AuthError::Configuration(format!("invalid auth token for header: {e}")))?;
    headers.insert(HeaderName::from_static("authorization"), value);
    Ok(headers)
}

/// Build a `HeaderMap` containing a single `x-api-key: {token}` entry.
///
/// Used by the Anthropic adapter for per-request auth injection.
///
/// # Errors
///
/// Returns `AuthError::Configuration` if the token contains characters
/// that are invalid in an HTTP header value.
pub fn api_key_header(token: &str) -> AuthResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value: HeaderValue = token
        .parse()
        .map_err(|e| AuthError::Configuration(format!("invalid auth token for header: {e}")))?;
    headers.insert(HeaderName::from_static("x-api-key"), value);
    Ok(headers)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    #[tokio::test]
    async fn static_key_returns_key() -> AuthResult<()> {
        let key = StaticKey::new("sk-test-123");
        assert_eq!(key.get_token().await?, "sk-test-123");
        Ok(())
    }

    #[tokio::test]
    async fn static_key_debug_redacts() {
        let key = StaticKey::new("sk-secret");
        let debug = format!("{key:?}");
        assert!(!debug.contains("sk-secret"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn oauth_credentials_serde_roundtrip() -> AuthResult<()> {
        let creds = OAuthCredentials {
            refresh_token: "rt-123".into(),
            access_token: "at-456".into(),
            expires_at: Some(1_700_000_000_000),
        };
        let json =
            serde_json::to_string(&creds).map_err(|e| AuthError::Configuration(e.to_string()))?;
        let parsed: OAuthCredentials =
            serde_json::from_str(&json).map_err(|e| AuthError::Configuration(e.to_string()))?;
        assert_eq!(parsed.refresh_token, "rt-123");
        assert_eq!(parsed.access_token, "at-456");
        assert_eq!(parsed.expires_at, Some(1_700_000_000_000));
        Ok(())
    }

    #[tokio::test]
    async fn oauth_token_returns_access_token_when_valid() -> AuthResult<()> {
        let creds = OAuthCredentials {
            refresh_token: "rt".into(),
            access_token: "valid-token".into(),
            expires_at: None, // non-expiring
        };
        let refresh_fn: RefreshFn = Arc::new(|_| Box::pin(async { panic!("should not refresh") }));
        let token = OAuthToken::new(creds, refresh_fn, None, None);
        assert_eq!(token.get_token().await?, "valid-token");
        Ok(())
    }

    #[tokio::test]
    async fn oauth_token_refreshes_when_expired() -> AuthResult<()> {
        let creds = OAuthCredentials {
            refresh_token: "rt-old".into(),
            access_token: "expired-token".into(),
            expires_at: Some(0), // already expired
        };
        let refresh_fn: RefreshFn = Arc::new(|old| {
            Box::pin(async move {
                assert_eq!(old.refresh_token, "rt-old");
                Ok(OAuthCredentials {
                    refresh_token: "rt-new".into(),
                    access_token: "fresh-token".into(),
                    expires_at: None,
                })
            })
        });
        let token = OAuthToken::new(creds, refresh_fn, None, None);
        assert_eq!(token.get_token().await?, "fresh-token");
        Ok(())
    }

    #[tokio::test]
    async fn oauth_token_calls_on_refresh() -> AuthResult<()> {
        let persist_count = Arc::new(AtomicU32::new(0));
        let persist_count_clone = persist_count.clone();

        let creds = OAuthCredentials {
            refresh_token: "rt".into(),
            access_token: "expired".into(),
            expires_at: Some(0),
        };
        let refresh_fn: RefreshFn = Arc::new(|_| {
            Box::pin(async {
                Ok(OAuthCredentials {
                    refresh_token: "rt-new".into(),
                    access_token: "new-token".into(),
                    expires_at: None,
                })
            })
        });
        let on_refresh: OnRefreshFn = Arc::new(move |creds| {
            let count = persist_count_clone.clone();
            let token = creds.access_token.clone();
            Box::pin(async move {
                assert_eq!(token, "new-token");
                count.fetch_add(1, Ordering::SeqCst);
            })
        });
        let token = OAuthToken::new(creds, refresh_fn, None, Some(on_refresh));
        let _ = token.get_token().await?;
        assert_eq!(persist_count.load(Ordering::SeqCst), 1);
        Ok(())
    }

    #[tokio::test]
    async fn oauth_token_uses_get_api_key_fn() -> AuthResult<()> {
        let creds = OAuthCredentials {
            refresh_token: "rt".into(),
            access_token: "raw-access".into(),
            expires_at: None,
        };
        let refresh_fn: RefreshFn = Arc::new(|_| Box::pin(async { panic!("should not refresh") }));
        let get_key: GetApiKeyFn = Arc::new(|creds| {
            let token = format!("transformed-{}", creds.access_token);
            Box::pin(async move { Ok(token) })
        });
        let token = OAuthToken::new(creds, refresh_fn, Some(get_key), None);
        assert_eq!(token.get_token().await?, "transformed-raw-access");
        Ok(())
    }

    #[tokio::test]
    async fn oauth_token_singleflight_refresh() -> AuthResult<()> {
        let refresh_count = Arc::new(AtomicU32::new(0));
        let refresh_count_clone = refresh_count.clone();

        let creds = OAuthCredentials {
            refresh_token: "rt".into(),
            access_token: "expired".into(),
            expires_at: Some(0),
        };
        let refresh_fn: RefreshFn = Arc::new(move |_| {
            let count = refresh_count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                // Simulate some async work
                tokio::task::yield_now().await;
                Ok(OAuthCredentials {
                    refresh_token: "rt-new".into(),
                    access_token: "fresh".into(),
                    expires_at: None,
                })
            })
        });

        let token = Arc::new(OAuthToken::new(creds, refresh_fn, None, None));

        // Spawn multiple concurrent get_token calls
        let mut handles = Vec::new();
        for _ in 0..5 {
            let t = token.clone();
            handles.push(tokio::spawn(async move { t.get_token().await }));
        }

        for handle in handles {
            let result = handle
                .await
                .map_err(|e| AuthError::Configuration(e.to_string()))??;
            assert_eq!(result, "fresh");
        }

        // Only one refresh should have occurred
        assert_eq!(refresh_count.load(Ordering::SeqCst), 1);
        Ok(())
    }

    #[tokio::test]
    async fn oauth_token_propagates_refresh_error() {
        let creds = OAuthCredentials {
            refresh_token: "rt".into(),
            access_token: "expired".into(),
            expires_at: Some(0),
        };
        let refresh_fn: RefreshFn = Arc::new(|_| {
            Box::pin(async { Err(AuthError::Authentication("refresh failed".into())) })
        });
        let token = OAuthToken::new(creds, refresh_fn, None, None);
        let result = token.get_token().await;
        assert!(result.is_err());
    }

    #[test]
    fn bearer_header_creates_valid_header() -> AuthResult<()> {
        let headers = bearer_header("my-token")?;
        let val = headers.get("authorization").expect("missing header");
        assert_eq!(val.to_str().unwrap_or(""), "Bearer my-token");
        Ok(())
    }

    #[test]
    fn api_key_header_creates_valid_header() -> AuthResult<()> {
        let headers = api_key_header("my-key")?;
        let val = headers.get("x-api-key").expect("missing header");
        assert_eq!(val.to_str().unwrap_or(""), "my-key");
        Ok(())
    }
}
