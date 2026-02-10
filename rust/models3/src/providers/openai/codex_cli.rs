//! Auto-detect Codex CLI OAuth credentials.
//!
//! Codex CLI stores `ChatGPT` OAuth credentials at `~/.codex/auth.json`.
//! This module reads those credentials so OpenAI models can be used
//! without an explicit `OPENAI_API_KEY`.

use std::sync::Arc;

use base64::{
    Engine as _,
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
};

use crate::auth::{AuthCredential, OAuthCredentials, OAuthToken, RefreshFn, StaticKey};
use crate::error::{ProviderDetails, SdkError};

// ---------------------------------------------------------------------------
// Constants (matching Codex CLI OAuth configuration)
// ---------------------------------------------------------------------------

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct CodexCliCredentials {
    oauth: OAuthCredentials,
    account_id: Option<String>,
    api_key: Option<String>,
    client_id: String,
}

#[derive(serde::Deserialize)]
struct StoredAuth {
    #[serde(rename = "OPENAI_API_KEY", default)]
    openai_api_key: Option<String>,
    tokens: Option<StoredTokens>,
}

#[derive(serde::Deserialize)]
struct StoredTokens {
    access_token: String,
    refresh_token: String,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

// ---------------------------------------------------------------------------
// Credential loading
// ---------------------------------------------------------------------------

/// Parse Codex auth JSON and extract OAuth credentials and metadata.
fn parse_credentials_json(data: &str) -> Option<CodexCliCredentials> {
    let parsed: StoredAuth = serde_json::from_str(data).ok()?;
    let tokens = parsed.tokens?;
    let api_key = parsed.openai_api_key.and_then(non_empty);

    if tokens.access_token.trim().is_empty() || tokens.refresh_token.trim().is_empty() {
        return None;
    }

    let account_id = tokens
        .account_id
        .filter(|id| !id.trim().is_empty())
        .or_else(|| tokens.id_token.as_deref().and_then(parse_account_id_claim));

    let client_id = tokens
        .id_token
        .as_deref()
        .and_then(parse_client_id_claim)
        .unwrap_or_else(|| CLIENT_ID.to_string());

    // Codex stores identity-scoped OAuth tokens. For OpenAI `/v1/responses`,
    // either an exchanged API key (OPENAI_API_KEY) must be present or the token
    // must already include API scopes.
    if api_key.is_none() && !has_openai_api_scope(&tokens.access_token) {
        return None;
    }

    Some(CodexCliCredentials {
        oauth: OAuthCredentials {
            refresh_token: tokens.refresh_token,
            access_token: tokens.access_token.clone(),
            expires_at: parse_exp_millis(&tokens.access_token),
        },
        account_id,
        api_key,
        client_id,
    })
}

/// Load OpenAI OAuth credentials from Codex CLI's auth file.
fn load_from_file() -> Option<CodexCliCredentials> {
    let home = directories::UserDirs::new()?.home_dir().to_path_buf();
    let path = home.join(".codex").join("auth.json");
    let data = std::fs::read_to_string(path).ok()?;
    parse_credentials_json(&data)
}

/// Load Codex CLI OAuth credentials for OpenAI API calls.
pub(crate) fn load_credentials() -> Option<CodexCliCredentials> {
    load_from_file()
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

fn refresh_fn(client_id: String) -> RefreshFn {
    Arc::new(move |old_creds| {
        let client_id = client_id.clone();
        Box::pin(async move {
            let current_refresh_token = old_creds.refresh_token.clone();

            let body = format!(
                "grant_type=refresh_token&refresh_token={}&client_id={}",
                url_encode(&current_refresh_token),
                url_encode(&client_id)
            );

            let response = reqwest::Client::new()
                .post(TOKEN_URL)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body)
                .send()
                .await
                .map_err(|e| SdkError::Authentication {
                    message: format!("Codex CLI token refresh failed: {e}"),
                    details: ProviderDetails::default(),
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(SdkError::Authentication {
                    message: format!("Codex CLI token refresh failed ({status}): {text}"),
                    details: ProviderDetails::default(),
                });
            }

            let token_data: TokenResponse =
                response
                    .json()
                    .await
                    .map_err(|e| SdkError::Authentication {
                        message: format!("failed to parse Codex CLI refresh response: {e}"),
                        details: ProviderDetails::default(),
                    })?;

            let expires_at = token_data
                .expires_in
                .map(|seconds| now_millis() + seconds * 1000)
                .or_else(|| parse_exp_millis(&token_data.access_token));

            Ok(OAuthCredentials {
                refresh_token: token_data.refresh_token.unwrap_or(current_refresh_token),
                access_token: token_data.access_token,
                expires_at,
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build an auth credential and optional `ChatGPT` account ID header value.
pub(crate) fn build_auth_credential(
    creds: CodexCliCredentials,
) -> (Arc<dyn AuthCredential>, Option<String>) {
    let CodexCliCredentials {
        oauth,
        account_id,
        api_key,
        client_id,
    } = creds;
    if let Some(api_key) = api_key {
        return (Arc::new(StaticKey::new(api_key)), account_id);
    }
    (
        Arc::new(OAuthToken::new(oauth, refresh_fn(client_id), None, None)),
        account_id,
    )
}

// ---------------------------------------------------------------------------
// JWT helpers
// ---------------------------------------------------------------------------

fn parse_exp_millis(token: &str) -> Option<u64> {
    decode_jwt_payload(token)?
        .get("exp")
        .and_then(serde_json::Value::as_u64)
        .map(|exp| exp.saturating_mul(1000))
}

fn parse_account_id_claim(token: &str) -> Option<String> {
    decode_jwt_payload(token)?
        .get("chatgpt_account_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn parse_client_id_claim(token: &str) -> Option<String> {
    let claims = decode_jwt_payload(token)?;
    let aud = claims.get("aud")?;

    if let Some(value) = aud.as_str() {
        return Some(value.to_string());
    }

    aud.as_array()
        .and_then(|values| values.first())
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn decode_jwt_payload(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .ok()?;
    serde_json::from_slice(&decoded).ok()
}

fn has_openai_api_scope(token: &str) -> bool {
    let Some(scp) = decode_jwt_payload(token).and_then(|payload| payload.get("scp").cloned())
    else {
        return false;
    };

    let Some(scopes) = scp.as_array() else {
        return false;
    };

    scopes.iter().any(|scope| {
        scope
            .as_str()
            .is_some_and(|value| value == "api.responses.write")
    })
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

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn url_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(char::from(HEX[usize::from(byte >> 4)]));
                encoded.push(char::from(HEX[usize::from(byte & 0x0F)]));
            }
        }
    }
    encoded
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn jwt(payload: &serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(payload.to_string());
        format!("{header}.{payload}.sig")
    }

    fn parse_or_panic(json: &str) -> CodexCliCredentials {
        match parse_credentials_json(json) {
            Some(creds) => creds,
            None => panic!("expected credentials to parse"),
        }
    }

    #[test]
    fn parse_valid_credentials() {
        let access_token = jwt(&serde_json::json!({
            "exp": 1_700_000_000_u64,
            "scp": ["openid", "api.responses.write"]
        }));
        let id_token = jwt(&serde_json::json!({
            "aud": "client-from-token",
            "chatgpt_account_id": "acct-from-token"
        }));
        let json = format!(
            r#"{{
                "OPENAI_API_KEY": null,
                "tokens": {{
                    "access_token": "{access_token}",
                    "refresh_token": "rt-123",
                    "id_token": "{id_token}",
                    "account_id": "acct-explicit"
                }},
                "last_refresh": "2026-01-01T00:00:00Z"
            }}"#
        );

        let creds = parse_or_panic(&json);
        assert_eq!(creds.oauth.access_token, access_token);
        assert_eq!(creds.oauth.refresh_token, "rt-123");
        assert_eq!(creds.oauth.expires_at, Some(1_700_000_000_000));
        assert_eq!(creds.account_id.as_deref(), Some("acct-explicit"));
        assert!(creds.api_key.is_none());
        assert_eq!(creds.client_id, "client-from-token");
    }

    #[test]
    fn parse_uses_id_token_account_id_when_field_missing() {
        let access_token = jwt(&serde_json::json!({
            "exp": 1_700_000_000_u64,
            "scp": ["api.responses.write"]
        }));
        let id_token = jwt(&serde_json::json!({
            "aud": "client-from-token",
            "chatgpt_account_id": "acct-from-token"
        }));
        let json = format!(
            r#"{{
                "tokens": {{
                    "access_token": "{access_token}",
                    "refresh_token": "rt-123",
                    "id_token": "{id_token}"
                }}
            }}"#
        );

        let creds = parse_or_panic(&json);
        assert_eq!(creds.account_id.as_deref(), Some("acct-from-token"));
        assert_eq!(creds.client_id, "client-from-token");
    }

    #[test]
    fn parse_uses_default_client_id_when_aud_missing() {
        let access_token = jwt(&serde_json::json!({
            "exp": 1_700_000_000_u64,
            "scp": ["api.responses.write"]
        }));
        let id_token = jwt(&serde_json::json!({ "chatgpt_account_id": "acct-from-token" }));
        let json = format!(
            r#"{{
                "tokens": {{
                    "access_token": "{access_token}",
                    "refresh_token": "rt-123",
                    "id_token": "{id_token}"
                }}
            }}"#
        );

        let creds = parse_or_panic(&json);
        assert_eq!(creds.client_id, CLIENT_ID);
    }

    #[test]
    fn parse_uses_first_aud_array_entry() {
        let access_token = jwt(&serde_json::json!({
            "exp": 1_700_000_000_u64,
            "scp": ["api.responses.write"]
        }));
        let id_token = jwt(&serde_json::json!({
            "aud": ["client-1", "client-2"]
        }));
        let json = format!(
            r#"{{
                "tokens": {{
                    "access_token": "{access_token}",
                    "refresh_token": "rt-123",
                    "id_token": "{id_token}"
                }}
            }}"#
        );

        let creds = parse_or_panic(&json);
        assert_eq!(creds.client_id, "client-1");
    }

    #[test]
    fn parse_missing_tokens_returns_none() {
        assert!(parse_credentials_json(r#"{ "last_refresh": "2026-01-01T00:00:00Z" }"#).is_none());
    }

    #[test]
    fn parse_missing_access_token_returns_none() {
        let json = r#"{
            "tokens": {
                "refresh_token": "rt-123"
            }
        }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_missing_refresh_token_returns_none() {
        let json = r#"{
            "tokens": {
                "access_token": "a.b.c"
            }
        }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_empty_tokens_returns_none() {
        let json = r#"{
            "tokens": {
                "access_token": "",
                "refresh_token": ""
            }
        }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_invalid_json_returns_none() {
        assert!(parse_credentials_json("not json").is_none());
    }

    #[test]
    fn parse_invalid_access_token_keeps_none_expiry() {
        let json = r#"{
            "tokens": {
                "access_token": "not-a-jwt",
                "refresh_token": "rt-123"
            }
        }"#;
        assert!(parse_credentials_json(json).is_none());
    }

    #[test]
    fn parse_uses_openai_api_key_even_without_api_scope() {
        let access_token = jwt(&serde_json::json!({
            "exp": 1_700_000_000_u64,
            "scp": ["openid", "profile", "email", "offline_access"]
        }));
        let id_token = jwt(&serde_json::json!({
            "aud": "client-from-token",
            "chatgpt_account_id": "acct-from-token"
        }));
        let json = format!(
            r#"{{
                "OPENAI_API_KEY": "sk-from-codex",
                "tokens": {{
                    "access_token": "{access_token}",
                    "refresh_token": "rt-123",
                    "id_token": "{id_token}"
                }}
            }}"#
        );

        let creds = parse_or_panic(&json);
        assert_eq!(creds.api_key.as_deref(), Some("sk-from-codex"));
    }

    #[test]
    fn parse_without_api_scope_and_without_api_key_returns_none() {
        let access_token = jwt(&serde_json::json!({
            "exp": 1_700_000_000_u64,
            "scp": ["openid", "profile", "email", "offline_access"]
        }));
        let id_token = jwt(&serde_json::json!({
            "aud": "client-from-token",
            "chatgpt_account_id": "acct-from-token"
        }));
        let json = format!(
            r#"{{
                "tokens": {{
                    "access_token": "{access_token}",
                    "refresh_token": "rt-123",
                    "id_token": "{id_token}"
                }}
            }}"#
        );
        assert!(parse_credentials_json(&json).is_none());
    }

    #[test]
    fn build_auth_credential_returns_credential() {
        let creds = CodexCliCredentials {
            oauth: OAuthCredentials {
                access_token: "at-test".into(),
                refresh_token: "rt-test".into(),
                expires_at: None,
            },
            account_id: Some("acct-123".into()),
            api_key: None,
            client_id: CLIENT_ID.into(),
        };

        let (_auth, account_id) = build_auth_credential(creds);
        assert_eq!(account_id.as_deref(), Some("acct-123"));
    }
}
