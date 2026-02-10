//! Anthropic OAuth login flow (Authorization Code + PKCE).
//!
//! Anthropic uses a manual code paste flow: the user opens a browser to the
//! authorization URL, completes login, copies the authorization code, and
//! pastes it back into the terminal.

use std::sync::Arc;

use eyre::{Result, bail, eyre};
use stencila_models3::auth::{OAuthCredentials, RefreshFn};

use crate::persist;
use crate::pkce;

// ---------------------------------------------------------------------------
// Constants (matching pi-mono's Anthropic OAuth configuration)
// ---------------------------------------------------------------------------

const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";
const SCOPES: &str = "user:inference user:mcp_servers user:profile user:sessions:claude_code";

// ---------------------------------------------------------------------------
// Token response types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

// ---------------------------------------------------------------------------
// Login flow
// ---------------------------------------------------------------------------

/// Perform the Anthropic OAuth login flow.
///
/// 1. Generates a PKCE challenge.
/// 2. Constructs the authorization URL and opens it in the browser.
/// 3. Prompts the user to paste the authorization code (format: `code#state`).
/// 4. Exchanges the code for tokens.
///
/// # Errors
///
/// Returns an error if the browser cannot be opened, the user provides
/// invalid input, or the token exchange fails.
pub async fn login() -> Result<OAuthCredentials> {
    let pkce = pkce::generate();

    // Build authorization URL
    let auth_url = format!(
        "{AUTHORIZE_URL}?{params}",
        params = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("code", "true")
            .append_pair("client_id", CLIENT_ID)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", SCOPES)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &pkce.verifier)
            .finish()
    );

    // Open browser
    tracing::info!("Opening browser for Anthropic OAuth");
    webbrowser::open(&auth_url).map_err(|e| eyre!("failed to open browser: {e}"))?;

    // Prompt user for the authorization code
    let input =
        stencila_ask::input("Paste the authorization code from the browser (format: code#state):")
            .await
            .map_err(|e| eyre!("failed to read authorization code: {e}"))?;

    // Parse code#state
    let (code, state) = parse_auth_code(&input)?;

    // Exchange code for tokens
    let credentials = exchange_code(&code, state.as_deref(), &pkce.verifier).await?;

    // Persist credentials
    persist::save_credentials("anthropic", &credentials)?;

    Ok(credentials)
}

/// Parse the authorization code input (format: `code#state`).
fn parse_auth_code(input: &str) -> Result<(String, Option<String>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("authorization code is empty");
    }

    if let Some((code, state)) = trimmed.split_once('#') {
        Ok((code.to_string(), Some(state.trim().to_string())))
    } else {
        Ok((trimmed.to_string(), None))
    }
}

/// Exchange an authorization code for tokens.
async fn exchange_code(
    code: &str,
    state: Option<&str>,
    verifier: &str,
) -> Result<OAuthCredentials> {
    let client = reqwest::Client::new();

    let mut body = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": CLIENT_ID,
        "code": code,
        "redirect_uri": REDIRECT_URI,
        "code_verifier": verifier,
    });

    if let Some(state_val) = state {
        body["state"] = serde_json::Value::String(state_val.to_string());
    }

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| eyre!("token exchange request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        bail!("token exchange failed ({status}): {text}");
    }

    let token_data: TokenResponse = response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse token response: {e}"))?;

    let now_ms = now_millis();
    let expires_at = now_ms + token_data.expires_in * 1000;

    Ok(OAuthCredentials {
        refresh_token: token_data.refresh_token,
        access_token: token_data.access_token,
        expires_at: Some(expires_at),
    })
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

/// Refresh an Anthropic OAuth token.
///
/// # Errors
///
/// Returns an error if the refresh request fails.
pub async fn refresh(old_credentials: OAuthCredentials) -> Result<OAuthCredentials> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "client_id": CLIENT_ID,
        "refresh_token": old_credentials.refresh_token,
    });

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| eyre!("Anthropic token refresh failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        bail!("Anthropic token refresh failed ({status}): {text}");
    }

    let token_data: TokenResponse = response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse refresh response: {e}"))?;

    let now_ms = now_millis();
    let expires_at = now_ms + token_data.expires_in * 1000;

    Ok(OAuthCredentials {
        refresh_token: token_data.refresh_token,
        access_token: token_data.access_token,
        expires_at: Some(expires_at),
    })
}

/// Build a [`RefreshFn`] for Anthropic tokens.
#[must_use]
pub fn refresh_fn() -> RefreshFn {
    Arc::new(|old_creds| Box::pin(async move { persist::to_sdk_result(refresh(old_creds).await) }))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code_with_state() -> Result<()> {
        let (code, state) = parse_auth_code("abc123#verifier456")?;
        assert_eq!(code, "abc123");
        assert_eq!(state.as_deref(), Some("verifier456"));
        Ok(())
    }

    #[test]
    fn parse_code_without_state() -> Result<()> {
        let (code, state) = parse_auth_code("abc123")?;
        assert_eq!(code, "abc123");
        assert!(state.is_none());
        Ok(())
    }

    #[test]
    fn parse_code_trims_whitespace() -> Result<()> {
        let (code, state) = parse_auth_code("  abc123#state  ")?;
        assert_eq!(code, "abc123");
        assert_eq!(state.as_deref(), Some("state"));
        Ok(())
    }

    #[test]
    fn parse_code_empty_fails() {
        assert!(parse_auth_code("").is_err());
        assert!(parse_auth_code("   ").is_err());
    }

    #[test]
    fn auth_url_contains_required_params() {
        let pkce = pkce::generate();
        let auth_url = format!(
            "{AUTHORIZE_URL}?{params}",
            params = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("client_id", CLIENT_ID)
                .append_pair("response_type", "code")
                .append_pair("redirect_uri", REDIRECT_URI)
                .append_pair("scope", SCOPES)
                .append_pair("code_challenge", &pkce.challenge)
                .append_pair("code_challenge_method", "S256")
                .finish()
        );

        assert!(auth_url.contains("client_id=9d1c250a"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("code_challenge_method=S256"));
        assert!(auth_url.contains("scope=user%3Ainference"));
    }
}
