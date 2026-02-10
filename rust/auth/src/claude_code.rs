//! Auto-detect Claude Code OAuth credentials.
//!
//! Claude Code stores OAuth credentials in the system keyring (service
//! `Claude Code-credentials`) and/or at `~/.claude/.credentials.json`.
//! This module checks both locations so that Anthropic models can be used
//! without an explicit `ANTHROPIC_API_KEY` when the user has an active
//! Claude Code subscription.

use std::sync::Arc;

use crate::{AuthCredential, AuthError, OAuthCredentials, OAuthToken, RefreshFn};

// ---------------------------------------------------------------------------
// Constants (matching pi-mono / oauth crate Anthropic configuration)
// ---------------------------------------------------------------------------

const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

/// Keyring service name used by Claude Code to store credentials.
const KEYRING_SERVICE: &str = "Claude Code-credentials";

/// Keyring account name used by Claude Code.
const KEYRING_ACCOUNT: &str = "default";

// ---------------------------------------------------------------------------
// Token refresh response
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

// ---------------------------------------------------------------------------
// Credential loading
// ---------------------------------------------------------------------------

/// Parse Claude Code's credentials JSON and extract the `claudeAiOauth` entry.
///
/// Returns `None` if the JSON is invalid or the required fields are missing.
fn parse_credentials_json(data: &str) -> Option<OAuthCredentials> {
    let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
    let oauth = parsed.get("claudeAiOauth")?;

    let access_token = oauth.get("accessToken")?.as_str()?.to_string();
    let refresh_token = oauth.get("refreshToken")?.as_str()?.to_string();
    let expires_at = oauth.get("expiresAt").and_then(serde_json::Value::as_u64);

    Some(OAuthCredentials {
        refresh_token,
        access_token,
        expires_at,
    })
}

/// Load Anthropic OAuth credentials from the system keyring.
///
/// Checks the `Claude Code-credentials` keyring service for stored
/// credentials. Returns `None` if the keyring is unavailable or does
/// not contain valid credentials.
fn load_from_keyring() -> Option<OAuthCredentials> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).ok()?;
    let data = entry.get_password().ok()?;
    parse_credentials_json(&data)
}

/// Load Anthropic OAuth credentials from Claude Code's credentials file.
///
/// Reads `~/.claude/.credentials.json` and extracts the `claudeAiOauth`
/// entry if present. Returns `None` if the file does not exist, cannot be
/// read, or does not contain valid credentials.
fn load_from_file() -> Option<OAuthCredentials> {
    let home = directories::UserDirs::new()?.home_dir().to_path_buf();
    let path = home.join(".claude").join(".credentials.json");
    let data = std::fs::read_to_string(path).ok()?;
    parse_credentials_json(&data)
}

/// Load Claude Code OAuth credentials, trying keyring first then file.
///
/// Checks the system keyring (`Claude Code-credentials` service), then
/// falls back to `~/.claude/.credentials.json`. Returns `None` if
/// neither source contains valid credentials.
pub fn load_credentials() -> Option<OAuthCredentials> {
    load_from_keyring().or_else(load_from_file)
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

/// Build a [`RefreshFn`] that refreshes Anthropic OAuth tokens using the
/// same token endpoint and client ID as Claude Code.
fn refresh_fn() -> RefreshFn {
    Arc::new(|old_creds| {
        Box::pin(async move {
            let client = reqwest::Client::new();

            let body = serde_json::json!({
                "grant_type": "refresh_token",
                "client_id": CLIENT_ID,
                "refresh_token": old_creds.refresh_token,
            });

            let response = client
                .post(TOKEN_URL)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    AuthError::Authentication(format!("Claude Code token refresh failed: {e}"))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(AuthError::Authentication(format!(
                    "Claude Code token refresh failed ({status}): {text}"
                )));
            }

            let token_data: TokenResponse = response.json().await.map_err(|e| {
                AuthError::Authentication(format!(
                    "failed to parse Claude Code refresh response: {e}"
                ))
            })?;

            let now_ms = now_millis();
            let expires_at = now_ms + token_data.expires_in * 1000;

            Ok(OAuthCredentials {
                refresh_token: token_data.refresh_token,
                access_token: token_data.access_token,
                expires_at: Some(expires_at),
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build an [`AuthCredential`] from Claude Code OAuth credentials.
///
/// Wraps the credentials in an [`OAuthToken`] that will automatically
/// refresh when expired.
#[must_use]
pub fn build_auth_credential(creds: OAuthCredentials) -> Arc<dyn AuthCredential> {
    Arc::new(OAuthToken::new(creds, refresh_fn(), None, None))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_credentials() {
        let json = r#"{
            "claudeAiOauth": {
                "accessToken": "at-123",
                "refreshToken": "rt-456",
                "expiresAt": 1700000000000
            }
        }"#;
        let creds = parse_credentials_json(json);
        assert!(creds.is_some());
        let creds = creds.expect("just checked is_some");
        assert_eq!(creds.access_token, "at-123");
        assert_eq!(creds.refresh_token, "rt-456");
        assert_eq!(creds.expires_at, Some(1_700_000_000_000));
    }

    #[test]
    fn parse_null_expires_at() {
        let json = r#"{
            "claudeAiOauth": {
                "accessToken": "at-123",
                "refreshToken": "rt-456",
                "expiresAt": null
            }
        }"#;
        let creds = parse_credentials_json(json);
        assert!(creds.is_some());
        let creds = creds.expect("just checked is_some");
        assert_eq!(creds.access_token, "at-123");
        assert_eq!(creds.expires_at, None);
    }

    #[test]
    fn parse_missing_expires_at() {
        let json = r#"{
            "claudeAiOauth": {
                "accessToken": "at-123",
                "refreshToken": "rt-456"
            }
        }"#;
        let creds = parse_credentials_json(json);
        assert!(creds.is_some());
        let creds = creds.expect("just checked is_some");
        assert_eq!(creds.expires_at, None);
    }

    #[test]
    fn parse_missing_access_token_returns_none() {
        let json = r#"{
            "claudeAiOauth": {
                "refreshToken": "rt-456",
                "expiresAt": 1700000000000
            }
        }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_missing_refresh_token_returns_none() {
        let json = r#"{
            "claudeAiOauth": {
                "accessToken": "at-123",
                "expiresAt": 1700000000000
            }
        }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_missing_oauth_section_returns_none() {
        let json = r#"{ "someOtherKey": {} }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_invalid_json_returns_none() {
        assert!(parse_credentials_json("not json").is_none());
    }

    #[test]
    fn parse_empty_string_returns_none() {
        assert!(parse_credentials_json("").is_none());
    }

    #[test]
    fn build_auth_credential_returns_credential() {
        let creds = OAuthCredentials {
            access_token: "at-test".into(),
            refresh_token: "rt-test".into(),
            expires_at: None,
        };
        let _auth = build_auth_credential(creds);
        // Just verify it constructs without panic
    }
}
