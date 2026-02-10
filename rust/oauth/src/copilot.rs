//! GitHub Copilot OAuth login flow (Device Code Grant).
//!
//! Uses the device code flow: the user is shown a URL and code,
//! opens the browser to verify, and the CLI polls for completion.

use std::sync::Arc;

use eyre::{Result, bail, eyre};
use serde::Deserialize;
use stencila_models3::auth::{OAuthCredentials, RefreshFn};

use crate::persist;

// ---------------------------------------------------------------------------
// Constants (matching pi-mono's GitHub Copilot OAuth configuration)
// ---------------------------------------------------------------------------

const CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";
const SCOPE: &str = "read:user";

/// Headers that mimic the VS Code Copilot Chat plugin.
fn copilot_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Ok(v) = "GitHubCopilotChat/0.35.0".parse() {
        headers.insert("user-agent", v);
    }
    if let Ok(v) = "vscode/1.107.0".parse() {
        headers.insert("editor-version", v);
    }
    if let Ok(v) = "copilot-chat/0.35.0".parse() {
        headers.insert("editor-plugin-version", v);
    }
    if let Ok(v) = "vscode-chat".parse() {
        headers.insert("copilot-integration-id", v);
    }
    headers
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Extended credentials that may include an enterprise domain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CopilotCredentials {
    /// Standard OAuth credentials.
    #[serde(flatten)]
    pub oauth: OAuthCredentials,
    /// Optional GitHub Enterprise domain (e.g. `company.ghe.com`).
    pub enterprise_domain: Option<String>,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
    expires_in: u64,
}

#[derive(Deserialize)]
struct DeviceTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    #[allow(dead_code)]
    interval: Option<u64>,
}

#[derive(Deserialize)]
struct CopilotTokenResponse {
    token: String,
    expires_at: u64,
}

// ---------------------------------------------------------------------------
// URL helpers
// ---------------------------------------------------------------------------

fn device_code_url(domain: &str) -> String {
    format!("https://{domain}/login/device/code")
}

fn access_token_url(domain: &str) -> String {
    format!("https://{domain}/login/oauth/access_token")
}

fn copilot_token_url(domain: &str) -> String {
    format!("https://api.{domain}/copilot_internal/v2/token")
}

/// Extract the API base URL from a Copilot token's `proxy-ep` field.
///
/// Token format: `tid=...;exp=...;proxy-ep=proxy.individual.githubcopilot.com;...`
/// Converts `proxy.xxx` to `api.xxx` and returns `https://api.xxx`.
fn base_url_from_token(token: &str) -> Option<String> {
    let proxy_ep = token
        .split(';')
        .find_map(|part| part.strip_prefix("proxy-ep="))?;
    let api_host = if let Some(rest) = proxy_ep.strip_prefix("proxy.") {
        format!("api.{rest}")
    } else {
        proxy_ep.to_string()
    };
    Some(format!("https://{api_host}"))
}

/// Get the Copilot API base URL from a token and/or enterprise domain.
#[must_use]
pub fn get_base_url(token: Option<&str>, enterprise_domain: Option<&str>) -> String {
    if let Some(t) = token
        && let Some(url) = base_url_from_token(t)
    {
        return url;
    }
    if let Some(domain) = enterprise_domain {
        return format!("https://copilot-api.{domain}");
    }
    "https://api.individual.githubcopilot.com".to_string()
}

/// Normalize a user-provided domain input.
#[must_use]
pub fn normalize_domain(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    url::Url::parse(&with_scheme)
        .ok()
        .map(|u| u.host_str().unwrap_or(trimmed).to_string())
}

// ---------------------------------------------------------------------------
// Login flow
// ---------------------------------------------------------------------------

/// Perform the GitHub Copilot OAuth login flow (device code grant).
///
/// 1. Optionally prompts for a GitHub Enterprise domain.
/// 2. Starts a device code flow.
/// 3. Shows the verification URL and user code.
/// 4. Polls for the GitHub access token.
/// 5. Exchanges the GitHub token for a Copilot token.
///
/// # Errors
///
/// Returns an error if the device flow fails, polling times out,
/// or the token exchange fails.
pub async fn login() -> Result<CopilotCredentials> {
    // Prompt for enterprise domain
    let domain_input = stencila_ask::input_with_default(
        "GitHub Enterprise URL/domain (blank for github.com):",
        "",
    )
    .await
    .map_err(|e| eyre!("failed to read enterprise domain: {e}"))?;

    let enterprise_domain = normalize_domain(&domain_input);
    let domain = enterprise_domain.as_deref().unwrap_or("github.com");

    // Start device flow
    let device = start_device_flow(domain).await?;

    // Show verification URL and code
    tracing::info!(
        "Visit {} and enter code: {}",
        device.verification_uri,
        device.user_code
    );

    // Open browser
    let _ = webbrowser::open(&device.verification_uri);

    // Wait for user to press enter (they need to go complete the flow)
    stencila_ask::wait_for_enter(&format!(
        "Enter code {} at {}, then press Enter",
        device.user_code, device.verification_uri
    ))
    .await
    .map_err(|e| eyre!("cancelled: {e}"))?;

    // Poll for access token
    let github_token = poll_for_access_token(
        domain,
        &device.device_code,
        device.interval,
        device.expires_in,
    )
    .await?;

    // Exchange GitHub token for Copilot token
    let credentials =
        exchange_for_copilot_token(&github_token, enterprise_domain.as_deref()).await?;

    // Persist credentials
    persist_copilot_credentials(&credentials)?;

    Ok(credentials)
}

/// Start the device code flow.
async fn start_device_flow(domain: &str) -> Result<DeviceCodeResponse> {
    let client = reqwest::Client::new();
    let url = device_code_url(domain);

    let body = serde_json::json!({
        "client_id": CLIENT_ID,
        "scope": SCOPE,
    });

    let response = client
        .post(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("User-Agent", "GitHubCopilotChat/0.35.0")
        .json(&body)
        .send()
        .await
        .map_err(|e| eyre!("device code request failed: {e}"))?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        bail!("device code request failed: {text}");
    }

    response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse device code response: {e}"))
}

/// Poll for the GitHub access token.
async fn poll_for_access_token(
    domain: &str,
    device_code: &str,
    interval_seconds: u64,
    expires_in: u64,
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = access_token_url(domain);
    let deadline = now_millis() + expires_in * 1000;
    let mut interval_ms = std::cmp::max(1000, interval_seconds * 1000);

    loop {
        if now_millis() >= deadline {
            bail!("device flow timed out");
        }

        let body = serde_json::json!({
            "client_id": CLIENT_ID,
            "device_code": device_code,
            "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
        });

        let response = client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", "GitHubCopilotChat/0.35.0")
            .json(&body)
            .send()
            .await
            .map_err(|e| eyre!("token poll request failed: {e}"))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            bail!("token poll failed: {text}");
        }

        let data: DeviceTokenResponse = response
            .json()
            .await
            .map_err(|e| eyre!("failed to parse poll response: {e}"))?;

        if let Some(token) = data.access_token {
            return Ok(token);
        }

        match data.error.as_deref() {
            Some("authorization_pending") | None => {
                tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
            }
            Some("slow_down") => {
                interval_ms += 5000;
                tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
            }
            Some(err) => bail!("device flow failed: {err}"),
        }
    }
}

/// Exchange a GitHub access token for a Copilot token.
async fn exchange_for_copilot_token(
    github_token: &str,
    enterprise_domain: Option<&str>,
) -> Result<CopilotCredentials> {
    let domain = enterprise_domain.unwrap_or("github.com");
    let url = copilot_token_url(domain);

    let client = reqwest::Client::new();
    let mut headers = copilot_headers();
    if let Ok(val) = "application/json".parse() {
        headers.insert("accept", val);
    }
    if let Ok(val) = format!("Bearer {github_token}").parse() {
        headers.insert(reqwest::header::AUTHORIZATION, val);
    }

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| eyre!("Copilot token exchange failed: {e}"))?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        bail!("Copilot token exchange failed: {text}");
    }

    let data: CopilotTokenResponse = response
        .json()
        .await
        .map_err(|e| eyre!("failed to parse Copilot token response: {e}"))?;

    let expires_at = data.expires_at * 1000;

    Ok(CopilotCredentials {
        oauth: OAuthCredentials {
            // GitHub access token is the "refresh token" for Copilot
            refresh_token: github_token.to_string(),
            access_token: data.token,
            expires_at: Some(expires_at),
        },
        enterprise_domain: enterprise_domain.map(String::from),
    })
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

/// Refresh a GitHub Copilot token.
///
/// Uses the stored GitHub access token (in `refresh_token` field) to
/// obtain a new Copilot token.
///
/// # Errors
///
/// Returns an error if the refresh request fails.
pub async fn refresh(
    old_credentials: OAuthCredentials,
    enterprise_domain: Option<&str>,
) -> Result<CopilotCredentials> {
    exchange_for_copilot_token(&old_credentials.refresh_token, enterprise_domain).await
}

/// Build a [`RefreshFn`] for Copilot tokens.
#[must_use]
pub fn refresh_fn(enterprise_domain: Option<&str>) -> RefreshFn {
    let domain = enterprise_domain.map(String::from);
    Arc::new(move |old_creds| {
        let domain = domain.clone();
        Box::pin(async move {
            let result = refresh(old_creds, domain.as_deref()).await;
            match result {
                Ok(copilot_creds) => {
                    if let Err(e) = persist_copilot_credentials(&copilot_creds) {
                        tracing::warn!("Failed to persist refreshed Copilot credentials: {e}");
                    }
                    Ok(copilot_creds.oauth)
                }
                Err(e) => Err(persist::to_auth_error(&e)),
            }
        })
    })
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn persist_copilot_credentials(creds: &CopilotCredentials) -> Result<()> {
    let name = persist::secret_name("copilot");
    let json = serde_json::to_string(creds)
        .map_err(|e| eyre!("failed to serialize Copilot credentials: {e}"))?;
    stencila_secrets::set(&name, &json)
        .map_err(|e| eyre!("failed to save Copilot credentials: {e}"))
}

/// Load persisted Copilot credentials (including enterprise domain).
///
/// # Errors
///
/// Returns an error if stored credentials cannot be deserialized.
pub fn load_credentials() -> Result<Option<CopilotCredentials>> {
    let name = persist::secret_name("copilot");
    match stencila_secrets::get_optional(&name) {
        Ok(Some(json)) => {
            let creds: CopilotCredentials = serde_json::from_str(&json)
                .map_err(|e| eyre!("failed to parse stored Copilot credentials: {e}"))?;
            Ok(Some(creds))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::warn!("Failed to read keyring for Copilot OAuth credentials: {e}");
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
    fn normalize_domain_standard() {
        assert_eq!(normalize_domain("github.com"), Some("github.com".into()));
        assert_eq!(
            normalize_domain("https://company.ghe.com"),
            Some("company.ghe.com".into())
        );
        assert_eq!(
            normalize_domain("company.ghe.com"),
            Some("company.ghe.com".into())
        );
    }

    #[test]
    fn normalize_domain_blank() {
        assert_eq!(normalize_domain(""), None);
        assert_eq!(normalize_domain("   "), None);
    }

    #[test]
    fn base_url_from_token_parses_proxy_ep() {
        let token = "tid=abc;exp=123;proxy-ep=proxy.individual.githubcopilot.com;sku=free";
        assert_eq!(
            base_url_from_token(token),
            Some("https://api.individual.githubcopilot.com".into())
        );
    }

    #[test]
    fn base_url_from_token_missing_proxy() {
        assert_eq!(base_url_from_token("tid=abc;exp=123"), None);
    }

    #[test]
    fn get_base_url_fallbacks() {
        // Token takes priority
        let token = "proxy-ep=proxy.enterprise.githubcopilot.com";
        assert_eq!(
            get_base_url(Some(token), Some("company.ghe.com")),
            "https://api.enterprise.githubcopilot.com"
        );

        // Enterprise domain fallback
        assert_eq!(
            get_base_url(None, Some("company.ghe.com")),
            "https://copilot-api.company.ghe.com"
        );

        // Default
        assert_eq!(
            get_base_url(None, None),
            "https://api.individual.githubcopilot.com"
        );
    }

    #[test]
    fn copilot_credentials_serde_roundtrip() -> Result<()> {
        let creds = CopilotCredentials {
            oauth: OAuthCredentials {
                refresh_token: "gho_abc123".into(),
                access_token: "copilot-token".into(),
                expires_at: Some(1_700_000_000_000),
            },
            enterprise_domain: Some("company.ghe.com".into()),
        };

        let json = serde_json::to_string(&creds).map_err(|e| eyre!("serialize: {e}"))?;
        let parsed: CopilotCredentials =
            serde_json::from_str(&json).map_err(|e| eyre!("deserialize: {e}"))?;

        assert_eq!(parsed.oauth.refresh_token, "gho_abc123");
        assert_eq!(parsed.enterprise_domain.as_deref(), Some("company.ghe.com"));
        Ok(())
    }
}
