//! Google Gemini OAuth login flow (Authorization Code + PKCE).
//!
//! Uses a local callback server on port 8085 to receive the authorization
//! code after the user completes login in the browser. After obtaining
//! tokens, discovers or provisions a Google Cloud Code Assist project.

use std::sync::Arc;

use eyre::{Result, bail, eyre};
use serde::Deserialize;

use crate::credentials::{GetApiKeyFn, OAuthCredentials, RefreshFn};
use crate::persist;
use crate::pkce;

// ---------------------------------------------------------------------------
// Constants (matching pi-mono's Gemini CLI OAuth configuration)
// ---------------------------------------------------------------------------

const CLIENT_ID: &str = "681255809395-oo8ft2oprdrmp9e3aqf6av3hmdi135j.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "GOCSBY-4uHgMPm-1o7Sk-geV6Cu5clXFsxl";
const REDIRECT_URI: &str = "http://localhost:8085/oauth2callback";
const CALLBACK_PORT: u16 = 8085;
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const CODE_ASSIST_ENDPOINT: &str = "https://cloudcode-pa.googleapis.com";

const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

/// Extended credentials that include the Google Cloud project ID.
///
/// Serialized to the keyring alongside the standard OAuth fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeminiCredentials {
    /// Standard OAuth credentials.
    #[serde(flatten)]
    pub oauth: OAuthCredentials,
    /// Google Cloud project ID for Cloud Code Assist.
    pub project_id: String,
}

#[derive(Deserialize)]
struct LoadCodeAssistResponse {
    #[serde(rename = "cloudaicompanionProject")]
    cloudaicompanion_project: Option<String>,
    #[serde(rename = "currentTier")]
    current_tier: Option<TierInfo>,
    #[serde(rename = "allowedTiers")]
    allowed_tiers: Option<Vec<TierInfo>>,
}

#[derive(Deserialize)]
struct TierInfo {
    id: Option<String>,
    #[serde(rename = "isDefault")]
    is_default: Option<bool>,
}

#[derive(Deserialize)]
struct OnboardResponse {
    name: Option<String>,
    done: Option<bool>,
    response: Option<OnboardResult>,
}

#[derive(Deserialize)]
struct OnboardResult {
    #[serde(rename = "cloudaicompanionProject")]
    cloudaicompanion_project: Option<ProjectInfo>,
}

#[derive(Deserialize)]
struct ProjectInfo {
    id: Option<String>,
}

const TIER_FREE: &str = "free-tier";
const TIER_LEGACY: &str = "legacy-tier";

// ---------------------------------------------------------------------------
// Login flow
// ---------------------------------------------------------------------------

/// Perform the Google Gemini OAuth login flow.
///
/// 1. Generates a PKCE challenge.
/// 2. Starts a local callback server on port 8085.
/// 3. Opens the browser to Google's authorization page.
/// 4. Waits for the callback with the authorization code.
/// 5. Exchanges the code for tokens.
/// 6. Discovers or provisions a Google Cloud project.
///
/// # Errors
///
/// Returns an error if the callback server cannot start, the browser
/// cannot be opened, or the token exchange/project discovery fails.
pub async fn login() -> Result<GeminiCredentials> {
    let pkce = pkce::generate();

    let scope = SCOPES.join(" ");
    let auth_url = format!(
        "{AUTH_URL}?{params}",
        params = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", CLIENT_ID)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", &scope)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &pkce.verifier)
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .finish()
    );

    // Open browser
    tracing::info!("Opening browser for Google Gemini OAuth");
    webbrowser::open(&auth_url).map_err(|e| eyre!("failed to open browser: {e}"))?;

    // Wait for callback
    tracing::debug!("Waiting for OAuth callback on port {CALLBACK_PORT}");
    let params = tokio::task::spawn_blocking(|| crate::callback::wait_for_callback(CALLBACK_PORT))
        .await
        .map_err(|e| eyre!("callback task failed: {e}"))?
        .map_err(|e| eyre!("{e}"))?;

    // Verify state (must be present and match to prevent CSRF)
    match params.state {
        Some(ref s) if s == &pkce.verifier => {}
        _ => bail!("OAuth state missing or mismatched — possible CSRF attack"),
    }

    // Exchange code for tokens
    let credentials = exchange_code(&params.code, &pkce.verifier).await?;

    if credentials.oauth.refresh_token.is_empty() {
        bail!("no refresh token received — please try again");
    }

    // Discover project
    let project_id = discover_project(&credentials.oauth.access_token).await?;

    let full_creds = GeminiCredentials {
        oauth: credentials.oauth,
        project_id,
    };

    // Persist as JSON
    persist_gemini_credentials(&full_creds)?;

    Ok(full_creds)
}

/// Exchange an authorization code for tokens.
async fn exchange_code(code: &str, verifier: &str) -> Result<GeminiCredentials> {
    let client = reqwest::Client::new();

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", CLIENT_ID)
        .append_pair("client_secret", CLIENT_SECRET)
        .append_pair("code", code)
        .append_pair("grant_type", "authorization_code")
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("code_verifier", verifier)
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

    Ok(GeminiCredentials {
        oauth: OAuthCredentials {
            refresh_token: token_data.refresh_token.unwrap_or_default(),
            access_token: token_data.access_token,
            expires_at: Some(expires_at),
        },
        project_id: String::new(),
    })
}

// ---------------------------------------------------------------------------
// Project discovery
// ---------------------------------------------------------------------------

/// Discover or provision a Google Cloud Code Assist project.
async fn discover_project(access_token: &str) -> Result<String> {
    // Check for user-provided project ID
    let env_project = std::env::var("GOOGLE_CLOUD_PROJECT")
        .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT_ID"))
        .ok();

    let client = reqwest::Client::new();
    let headers = build_google_headers(access_token);

    // Try to load existing project
    let load_body = serde_json::json!({
        "cloudaicompanionProject": env_project,
        "metadata": {
            "ideType": "IDE_UNSPECIFIED",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI",
            "duetProject": env_project,
        }
    });

    let load_url = format!("{CODE_ASSIST_ENDPOINT}/v1internal:loadCodeAssist");
    let load_response = client
        .post(&load_url)
        .headers(headers.clone())
        .json(&load_body)
        .send()
        .await
        .map_err(|e| eyre!("loadCodeAssist request failed: {e}"))?;

    if load_response.status().is_success() {
        let data: LoadCodeAssistResponse = load_response
            .json()
            .await
            .map_err(|e| eyre!("failed to parse loadCodeAssist response: {e}"))?;

        if data.current_tier.is_some() {
            if let Some(project) = data.cloudaicompanion_project {
                return Ok(project);
            }
            if let Some(project) = env_project {
                return Ok(project);
            }
            bail!(
                "This account requires setting the GOOGLE_CLOUD_PROJECT or \
                 GOOGLE_CLOUD_PROJECT_ID environment variable"
            );
        }

        // Need to onboard — determine tier
        let tier_id = data
            .allowed_tiers
            .as_ref()
            .and_then(|tiers| tiers.iter().find(|t| t.is_default == Some(true)))
            .and_then(|t| t.id.as_deref())
            .unwrap_or(TIER_LEGACY);

        if tier_id != TIER_FREE {
            if let Some(project) = env_project {
                return Ok(project);
            }
            bail!(
                "This account requires setting the GOOGLE_CLOUD_PROJECT or \
                 GOOGLE_CLOUD_PROJECT_ID environment variable"
            );
        }

        return onboard_user(access_token, TIER_FREE, env_project.as_deref()).await;
    }

    // Load failed — try onboarding with free tier
    onboard_user(access_token, TIER_FREE, env_project.as_deref()).await
}

/// Onboard a new user and provision a project.
async fn onboard_user(
    access_token: &str,
    tier_id: &str,
    env_project: Option<&str>,
) -> Result<String> {
    let client = reqwest::Client::new();
    let headers = build_google_headers(access_token);

    let mut body = serde_json::json!({
        "tierId": tier_id,
        "metadata": {
            "ideType": "IDE_UNSPECIFIED",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI",
        }
    });

    if tier_id != TIER_FREE
        && let Some(project) = env_project
    {
        body["cloudaicompanionProject"] = serde_json::Value::String(project.to_string());
        body["metadata"]["duetProject"] = serde_json::Value::String(project.to_string());
    }

    let onboard_url = format!("{CODE_ASSIST_ENDPOINT}/v1internal:onboardUser");
    let response = client
        .post(&onboard_url)
        .headers(headers.clone())
        .json(&body)
        .send()
        .await
        .map_err(|e| eyre!("onboardUser request failed: {e}"))?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        bail!("onboardUser failed: {text}");
    }

    let mut lro: OnboardResponse = response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse onboardUser response: {e}"))?;

    // Poll if not done
    if lro.done != Some(true)
        && let Some(ref op_name) = lro.name
    {
        lro = poll_operation(op_name, &headers).await?;
    }

    // Extract project ID
    if let Some(project_id) = lro
        .response
        .and_then(|r| r.cloudaicompanion_project)
        .and_then(|p| p.id)
    {
        return Ok(project_id);
    }

    if let Some(project) = env_project {
        return Ok(project.to_string());
    }

    bail!(
        "Could not discover or provision a Google Cloud project. \
         Try setting GOOGLE_CLOUD_PROJECT or GOOGLE_CLOUD_PROJECT_ID."
    )
}

/// Poll a long-running operation until completion.
async fn poll_operation(
    operation_name: &str,
    headers: &reqwest::header::HeaderMap,
) -> Result<OnboardResponse> {
    let client = reqwest::Client::new();
    let url = format!("{CODE_ASSIST_ENDPOINT}/v1internal/{operation_name}");

    for attempt in 0..60 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        let response = client
            .get(&url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| eyre!("failed to poll operation: {e}"))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            bail!("operation poll failed: {text}");
        }

        let data: OnboardResponse = response
            .json()
            .await
            .map_err(|e| eyre!("failed to parse operation response: {e}"))?;

        if data.done == Some(true) {
            return Ok(data);
        }
    }

    bail!("project provisioning timed out after 5 minutes")
}

fn build_google_headers(access_token: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Ok(val) = format!("Bearer {access_token}").parse() {
        headers.insert(reqwest::header::AUTHORIZATION, val);
    }
    if let Ok(val) = "application/json".parse() {
        headers.insert(reqwest::header::CONTENT_TYPE, val);
    }
    headers
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

/// Refresh a Google Cloud OAuth token.
///
/// # Errors
///
/// Returns an error if the refresh request fails.
pub async fn refresh(
    old_credentials: OAuthCredentials,
    project_id: &str,
) -> Result<GeminiCredentials> {
    let client = reqwest::Client::new();

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", CLIENT_ID)
        .append_pair("client_secret", CLIENT_SECRET)
        .append_pair("refresh_token", &old_credentials.refresh_token)
        .append_pair("grant_type", "refresh_token")
        .finish();

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| eyre!("Google Cloud token refresh failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        bail!("Google Cloud token refresh failed ({status}): {text}");
    }

    let token_data: TokenResponse = response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse refresh response: {e}"))?;

    let now_ms = now_millis();
    let expires_at = now_ms + token_data.expires_in * 1000;

    Ok(GeminiCredentials {
        oauth: OAuthCredentials {
            refresh_token: token_data
                .refresh_token
                .unwrap_or(old_credentials.refresh_token),
            access_token: token_data.access_token,
            expires_at: Some(expires_at),
        },
        project_id: project_id.to_string(),
    })
}

/// Build a [`RefreshFn`] for Gemini tokens.
///
/// The project ID is captured so that refreshed credentials maintain
/// the same project association.
#[must_use]
pub fn refresh_fn(project_id: &str) -> RefreshFn {
    let project_id = project_id.to_string();
    Arc::new(move |old_creds| {
        let pid = project_id.clone();
        Box::pin(async move {
            let result = refresh(old_creds, &pid).await;
            match result {
                Ok(gemini_creds) => {
                    // Also persist the updated credentials
                    if let Err(e) = persist_gemini_credentials(&gemini_creds) {
                        tracing::warn!("Failed to persist refreshed Gemini credentials: {e}");
                    }
                    Ok(gemini_creds.oauth)
                }
                Err(e) => Err(persist::to_auth_error(&e)),
            }
        })
    })
}

/// Build a [`GetApiKeyFn`] that returns JSON `{"token": ..., "projectId": ...}`.
///
/// Gemini requires both the access token and project ID for API calls.
#[must_use]
pub fn get_api_key_fn(project_id: &str) -> GetApiKeyFn {
    let project_id = project_id.to_string();
    Arc::new(move |creds: &OAuthCredentials| {
        let json = serde_json::json!({
            "token": creds.access_token,
            "projectId": project_id,
        });
        let result = json.to_string();
        Box::pin(async move { Ok(result) })
    })
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

/// Persist Gemini credentials (including project ID) to the keyring.
fn persist_gemini_credentials(creds: &GeminiCredentials) -> Result<()> {
    let name = persist::secret_name("gemini");
    let json = serde_json::to_string(creds)
        .map_err(|e| eyre!("failed to serialize Gemini credentials: {e}"))?;
    stencila_secrets::set(&name, &json).map_err(|e| eyre!("failed to save Gemini credentials: {e}"))
}

/// Load persisted Gemini credentials (including project ID).
///
/// # Errors
///
/// Returns an error if stored credentials cannot be deserialized.
pub fn load_credentials() -> Result<Option<GeminiCredentials>> {
    let name = persist::secret_name("gemini");
    match stencila_secrets::get_optional(&name) {
        Ok(Some(json)) => {
            let creds: GeminiCredentials = serde_json::from_str(&json)
                .map_err(|e| eyre!("failed to parse stored Gemini credentials: {e}"))?;
            Ok(Some(creds))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::warn!("Failed to read keyring for Gemini OAuth credentials: {e}");
            Ok(None)
        }
    }
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
    fn auth_url_contains_required_params() {
        let pkce = pkce::generate();
        let scope = SCOPES.join(" ");
        let auth_url = format!(
            "{AUTH_URL}?{params}",
            params = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("client_id", CLIENT_ID)
                .append_pair("response_type", "code")
                .append_pair("redirect_uri", REDIRECT_URI)
                .append_pair("scope", &scope)
                .append_pair("code_challenge", &pkce.challenge)
                .append_pair("code_challenge_method", "S256")
                .append_pair("access_type", "offline")
                .finish()
        );

        assert!(auth_url.contains("client_id=681255809395"));
        assert!(auth_url.contains("redirect_uri=http"));
        assert!(auth_url.contains("8085"));
        assert!(auth_url.contains("cloud-platform"));
        assert!(auth_url.contains("access_type=offline"));
    }

    #[test]
    fn gemini_credentials_serde_roundtrip() -> Result<()> {
        let creds = GeminiCredentials {
            oauth: OAuthCredentials {
                refresh_token: "rt-123".into(),
                access_token: "at-456".into(),
                expires_at: Some(1_700_000_000_000),
            },
            project_id: "my-project-id".into(),
        };

        let json = serde_json::to_string(&creds).map_err(|e| eyre!("serialize: {e}"))?;
        let parsed: GeminiCredentials =
            serde_json::from_str(&json).map_err(|e| eyre!("deserialize: {e}"))?;

        assert_eq!(parsed.oauth.refresh_token, "rt-123");
        assert_eq!(parsed.project_id, "my-project-id");
        Ok(())
    }
}
