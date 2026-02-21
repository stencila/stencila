//! OpenAI OAuth login flow (Authorization Code + PKCE).
//!
//! Uses a local callback server on port 1455 to receive the authorization
//! code after the user completes login in the browser. Decodes the JWT
//! access token to extract the ChatGPT account ID.

use std::sync::Arc;

use eyre::{Result, bail, eyre};
use serde::Deserialize;

use crate::credentials::{OAuthCredentials, RefreshFn};
use crate::persist;
use crate::pkce;

// ---------------------------------------------------------------------------
// Constants (matching pi-mono's OpenAI Codex OAuth configuration)
// ---------------------------------------------------------------------------

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const CALLBACK_PORT: u16 = 1455;
const SCOPE: &str = "openid profile email offline_access";
const JWT_CLAIM_PATH: &str = "https://api.openai.com/auth";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

/// Extended credentials that include the `ChatGPT` account ID.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenAICredentials {
    /// Standard OAuth credentials.
    #[serde(flatten)]
    pub oauth: OAuthCredentials,
    /// `ChatGPT` account ID extracted from the JWT.
    pub account_id: String,
}

// ---------------------------------------------------------------------------
// Login flow
// ---------------------------------------------------------------------------

/// Perform the `OpenAI` Codex OAuth login flow.
///
/// 1. Generates a PKCE challenge.
/// 2. Starts a local callback server on port 1455.
/// 3. Opens the browser to the `OpenAI` authorization page.
/// 4. Waits for the callback (or falls back to manual paste).
/// 5. Exchanges the code for tokens.
/// 6. Decodes the JWT to extract the account ID.
///
/// # Errors
///
/// Returns an error if the callback server cannot start, the browser
/// cannot be opened, or the token exchange fails.
pub async fn login() -> Result<OpenAICredentials> {
    let pkce = pkce::generate();
    let state = generate_state();

    let auth_url = format!(
        "{AUTHORIZE_URL}?{params}",
        params = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("response_type", "code")
            .append_pair("client_id", CLIENT_ID)
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", SCOPE)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &state)
            .append_pair("id_token_add_organizations", "true")
            .append_pair("codex_cli_simplified_flow", "true")
            .finish()
    );

    // Open browser
    tracing::info!("Opening browser for OpenAI OAuth");
    webbrowser::open(&auth_url).map_err(|e| eyre!("failed to open browser: {e}"))?;

    // Wait for callback
    tracing::debug!("Waiting for OAuth callback on port {CALLBACK_PORT}");
    let params_result =
        tokio::task::spawn_blocking(|| crate::callback::wait_for_callback(CALLBACK_PORT))
            .await
            .map_err(|e| eyre!("callback task failed: {e}"))?;

    let code = match params_result {
        Ok(params) => {
            // Verify state (must be present and match to prevent CSRF)
            match params.state {
                Some(ref s) if s == &state => {}
                _ => bail!("OAuth state missing or mismatched — possible CSRF attack"),
            }
            params.code
        }
        Err(crate::callback::CallbackError::OAuthDenied { error, description }) => {
            // The user explicitly denied authorization or the provider rejected it.
            // Do NOT fall back to manual paste — surface the error directly.
            bail!("OAuth authorization denied: {error} - {description}");
        }
        Err(crate::callback::CallbackError::Transport(e)) => {
            // Transport/setup failure — fall back to manual paste.
            tracing::warn!("Callback server failed ({e}), falling back to manual paste");
            let input = stencila_ask::input("Paste the authorization code (or full redirect URL):")
                .await
                .map_err(|e| eyre!("failed to read authorization code: {e}"))?;
            parse_authorization_input(&input, &state)?
        }
    };

    // Exchange code for tokens
    let credentials = exchange_code(&code, &pkce.verifier).await?;

    // Extract account ID from JWT
    let account_id = get_account_id(&credentials.oauth.access_token)
        .ok_or_else(|| eyre!("failed to extract account ID from access token JWT"))?;

    let full_creds = OpenAICredentials {
        oauth: credentials.oauth,
        account_id,
    };

    // Persist credentials
    persist_openai_credentials(&full_creds)?;

    Ok(full_creds)
}

/// Exchange an authorization code for tokens.
async fn exchange_code(code: &str, verifier: &str) -> Result<OpenAICredentials> {
    let client = reqwest::Client::new();

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("code", code)
        .append_pair("code_verifier", verifier)
        .append_pair("redirect_uri", REDIRECT_URI)
        .finish();

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
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

    Ok(OpenAICredentials {
        oauth: OAuthCredentials {
            refresh_token: token_data.refresh_token,
            access_token: token_data.access_token,
            expires_at: Some(expires_at),
        },
        account_id: String::new(),
    })
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

/// Refresh an `OpenAI` Codex OAuth token.
///
/// # Errors
///
/// Returns an error if the refresh request fails.
pub async fn refresh(old_credentials: OAuthCredentials) -> Result<OpenAICredentials> {
    let client = reqwest::Client::new();

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "refresh_token")
        .append_pair("refresh_token", &old_credentials.refresh_token)
        .append_pair("client_id", CLIENT_ID)
        .finish();

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| eyre!("OpenAI token refresh failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        bail!("OpenAI token refresh failed ({status}): {text}");
    }

    let token_data: TokenResponse = response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse refresh response: {e}"))?;

    let account_id = get_account_id(&token_data.access_token)
        .ok_or_else(|| eyre!("failed to extract account ID from refreshed token JWT"))?;

    let now_ms = now_millis();
    let expires_at = now_ms + token_data.expires_in * 1000;

    Ok(OpenAICredentials {
        oauth: OAuthCredentials {
            refresh_token: token_data.refresh_token,
            access_token: token_data.access_token,
            expires_at: Some(expires_at),
        },
        account_id,
    })
}

/// Build a [`RefreshFn`] for `OpenAI` tokens.
#[must_use]
pub fn refresh_fn() -> RefreshFn {
    Arc::new(|old_creds| {
        Box::pin(async move {
            let result = refresh(old_creds).await;
            match result {
                Ok(openai_creds) => {
                    if let Err(e) = persist_openai_credentials(&openai_creds) {
                        tracing::warn!("Failed to persist refreshed OpenAI credentials: {e}");
                    }
                    Ok(openai_creds.oauth)
                }
                Err(e) => Err(persist::to_auth_error(&e)),
            }
        })
    })
}

// ---------------------------------------------------------------------------
// JWT decoding
// ---------------------------------------------------------------------------

/// Decode a JWT access token and extract the `ChatGPT` account ID.
///
/// The account ID is in the claim at `https://api.openai.com/auth`.
fn get_account_id(access_token: &str) -> Option<String> {
    let parts: Vec<&str> = access_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode base64url payload (part[1])
    let payload = parts.get(1)?;
    let decoded = base64_url_decode(payload)?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    let auth = json.get(JWT_CLAIM_PATH)?;
    let account_id = auth.get("chatgpt_account_id")?.as_str()?;

    if account_id.is_empty() {
        None
    } else {
        Some(account_id.to_string())
    }
}

/// Decode a base64url-encoded string (no padding).
fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.decode(input).ok()
}

// ---------------------------------------------------------------------------
// Input parsing
// ---------------------------------------------------------------------------

/// Parse user input that may be a full URL, query params, `code#state`, or just a code.
fn parse_authorization_input(input: &str, expected_state: &str) -> Result<String> {
    let value = input.trim();
    if value.is_empty() {
        bail!("authorization code is empty");
    }

    // Try as full URL
    if let Ok(url) = url::Url::parse(value) {
        let code = url
            .query_pairs()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.to_string());
        let state = url
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.to_string());

        if let Some(s) = &state
            && s != expected_state
        {
            bail!("state mismatch");
        }
        if let Some(c) = code {
            return Ok(c);
        }
    }

    // Try code#state format
    if value.contains('#') {
        let parts: Vec<&str> = value.splitn(2, '#').collect();
        if parts.len() == 2 {
            if let Some(state) = parts.get(1)
                && !state.is_empty()
                && *state != expected_state
            {
                bail!("state mismatch");
            }
            if let Some(code) = parts.first() {
                return Ok((*code).to_string());
            }
        }
    }

    // Try query params
    if value.contains("code=") {
        let params: Vec<(String, String)> = url::form_urlencoded::parse(value.as_bytes())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let code = params.iter().find(|(k, _)| k == "code").map(|(_, v)| v);
        let state = params.iter().find(|(k, _)| k == "state").map(|(_, v)| v);

        if let Some(s) = state
            && s != expected_state
        {
            bail!("state mismatch");
        }
        if let Some(c) = code {
            return Ok(c.clone());
        }
    }

    // Treat as raw code
    Ok(value.to_string())
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn persist_openai_credentials(creds: &OpenAICredentials) -> Result<()> {
    let name = persist::secret_name("openai");
    let json = serde_json::to_string(creds)
        .map_err(|e| eyre!("failed to serialize OpenAI credentials: {e}"))?;
    stencila_secrets::set(&name, &json).map_err(|e| eyre!("failed to save OpenAI credentials: {e}"))
}

/// Load persisted `OpenAI` credentials (including account ID).
///
/// # Errors
///
/// Returns an error if stored credentials cannot be deserialized.
pub fn load_credentials() -> Result<Option<OpenAICredentials>> {
    let name = persist::secret_name("openai");
    match stencila_secrets::get_optional(&name) {
        Ok(Some(json)) => {
            let creds: OpenAICredentials = serde_json::from_str(&json)
                .map_err(|e| eyre!("failed to parse stored OpenAI credentials: {e}"))?;
            Ok(Some(creds))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::warn!("Failed to read keyring for OpenAI OAuth credentials: {e}");
            Ok(None)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate_state() -> String {
    use rand::RngExt;
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);
    hex::encode(bytes)
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

// Inline hex encoding to avoid another dependency
mod hex {
    use std::fmt::Write;

    pub fn encode(bytes: [u8; 16]) -> String {
        bytes.iter().fold(String::with_capacity(32), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_account_id_extraction() {
        // Build a fake JWT with the expected claim structure
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let header = URL_SAFE_NO_PAD.encode(b"{}");
        let payload_json = serde_json::json!({
            JWT_CLAIM_PATH: {
                "chatgpt_account_id": "acct_123abc"
            }
        });
        let payload = URL_SAFE_NO_PAD.encode(payload_json.to_string().as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(b"sig");

        let token = format!("{header}.{payload}.{signature}");
        assert_eq!(get_account_id(&token), Some("acct_123abc".into()));
    }

    #[test]
    fn jwt_missing_claim_returns_none() {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let header = URL_SAFE_NO_PAD.encode(b"{}");
        let payload = URL_SAFE_NO_PAD.encode(b"{}");
        let signature = URL_SAFE_NO_PAD.encode(b"sig");

        let token = format!("{header}.{payload}.{signature}");
        assert_eq!(get_account_id(&token), None);
    }

    #[test]
    fn jwt_invalid_token_returns_none() {
        assert_eq!(get_account_id("not-a-jwt"), None);
        assert_eq!(get_account_id("a.b"), None);
    }

    #[test]
    fn parse_url_input() -> Result<()> {
        let state = "mystate123";
        let code = parse_authorization_input(
            "http://localhost:1455/auth/callback?code=abc&state=mystate123",
            state,
        )?;
        assert_eq!(code, "abc");
        Ok(())
    }

    #[test]
    fn parse_code_hash_state() -> Result<()> {
        let code = parse_authorization_input("abc#mystate123", "mystate123")?;
        assert_eq!(code, "abc");
        Ok(())
    }

    #[test]
    fn parse_raw_code() -> Result<()> {
        let code = parse_authorization_input("abc123", "anystate")?;
        assert_eq!(code, "abc123");
        Ok(())
    }

    #[test]
    fn parse_state_mismatch_fails() {
        let result =
            parse_authorization_input("http://localhost:1455?code=abc&state=wrong", "expected");
        assert!(result.is_err());
    }

    #[test]
    fn auth_url_contains_required_params() {
        let pkce = pkce::generate();
        let auth_url = format!(
            "{AUTHORIZE_URL}?{params}",
            params = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("response_type", "code")
                .append_pair("client_id", CLIENT_ID)
                .append_pair("redirect_uri", REDIRECT_URI)
                .append_pair("scope", SCOPE)
                .append_pair("code_challenge", &pkce.challenge)
                .append_pair("code_challenge_method", "S256")
                .finish()
        );

        assert!(auth_url.contains("client_id=app_EMoamEEZ73f0CkXaXp7hrann"));
        assert!(auth_url.contains("1455"));
        assert!(auth_url.contains("offline_access"));
    }

    #[test]
    fn openai_credentials_serde_roundtrip() -> Result<()> {
        let creds = OpenAICredentials {
            oauth: OAuthCredentials {
                refresh_token: "rt-123".into(),
                access_token: "at-456".into(),
                expires_at: Some(1_700_000_000_000),
            },
            account_id: "acct_abc123".into(),
        };

        let json = serde_json::to_string(&creds).map_err(|e| eyre!("serialize: {e}"))?;
        let parsed: OpenAICredentials =
            serde_json::from_str(&json).map_err(|e| eyre!("deserialize: {e}"))?;

        assert_eq!(parsed.oauth.refresh_token, "rt-123");
        assert_eq!(parsed.account_id, "acct_abc123");
        Ok(())
    }
}
