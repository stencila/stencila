use std::{num::NonZeroU32, sync::LazyLock};

use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use itertools::Itertools;
use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderName, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::time::Instant;

use stencila_codec::eyre::{Result, bail};
use stencila_secrets::GITHUB_TOKEN;
use stencila_version::STENCILA_USER_AGENT;

const API_BASE_URL: &str = "https://api.github.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitHubAuthPolicy {
    PreferUser,
    #[allow(dead_code)]
    PreferRepoInstallation,
}

// Rate limits based on GitHub API documentation with conservative margins
//
// https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api?apiVersion=2022-11-28
// https://docs.github.com/en/rest/search/search?apiVersion=2022-11-28#rate-limit

// Default API rate limiters
static DEFAULT_UNAUTH_GOVERNOR: LazyLock<
    GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
> = LazyLock::new(|| {
    // 60 requests/hour => 58 requests/hour
    GovernorRateLimiter::direct(Quota::per_hour(NonZeroU32::new(58).expect("invalid")))
});

static DEFAULT_AUTH_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 5000 requests/hour => 4900 requests/hour
        GovernorRateLimiter::direct(Quota::per_hour(NonZeroU32::new(4900).expect("invalid")))
    });

// Search code rate limiter (requires authentication)
static SEARCH_CODE_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 10 requests/minute => 9 requests/minute
        GovernorRateLimiter::direct(Quota::per_minute(NonZeroU32::new(9).expect("invalid")))
    });

// Other search endpoints rate limiters
static SEARCH_UNAUTH_GOVERNOR: LazyLock<
    GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
> = LazyLock::new(|| {
    // 10 requests/minute => 9 requests/minute
    GovernorRateLimiter::direct(Quota::per_minute(NonZeroU32::new(9).expect("invalid")))
});

static SEARCH_AUTH_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 30 requests/minute => 28 requests/minute
        GovernorRateLimiter::direct(Quota::per_minute(NonZeroU32::new(28).expect("invalid")))
    });

/// Apply rate limiting based on URL and authentication status
pub(crate) async fn apply_rate_limiting(url: &str, authenticated: bool) -> Result<()> {
    let start = Instant::now();

    if url.contains("/search/code") {
        if !authenticated {
            bail!("GitHub search/code endpoint requires authentication");
        }
        SEARCH_CODE_GOVERNOR.until_ready().await;
    } else if url.contains("/search/") {
        if authenticated {
            SEARCH_AUTH_GOVERNOR.until_ready().await;
        } else {
            SEARCH_UNAUTH_GOVERNOR.until_ready().await;
        }
    } else if authenticated {
        DEFAULT_AUTH_GOVERNOR.until_ready().await;
    } else {
        DEFAULT_UNAUTH_GOVERNOR.until_ready().await;
    }

    let duration = (Instant::now() - start).as_millis();
    if duration > 0 {
        tracing::trace!("Rate limited for {duration}ms");
    }

    Ok(())
}

/// Get GitHub API token with optional repo-specific resolution
///
/// When `owner` and `repo` are provided, token resolution order depends on
/// `policy` after first checking the local `GITHUB_TOKEN` env var / keyring.
///
/// When not provided (for search/general APIs), just tries:
/// 1. GITHUB_TOKEN env var / keyring
pub(crate) async fn get_token(
    owner: Option<&str>,
    repo: Option<&str>,
    policy: GitHubAuthPolicy,
) -> Option<String> {
    // First try local secret/env var
    if let Ok(token) = stencila_secrets::env_or_get(GITHUB_TOKEN) {
        return Some(token);
    }

    // If repo context provided, try Cloud tokens
    if let (Some(owner), Some(repo)) = (owner, repo) {
        let user_token = async { stencila_cloud::get_token("github").await.ok() };
        let repo_token = async { stencila_cloud::get_repo_token(owner, repo).await.ok() };

        match policy {
            GitHubAuthPolicy::PreferUser => {
                if let Some(token) = user_token.await {
                    return Some(token);
                }
                if let Some(token) = repo_token.await {
                    return Some(token);
                }
            }
            GitHubAuthPolicy::PreferRepoInstallation => {
                if let Some(token) = repo_token.await {
                    return Some(token);
                }
                if let Some(token) = user_token.await {
                    return Some(token);
                }
            }
        }

        return None;
    }

    None
}

pub(crate) static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .default_headers(HeaderMap::from_iter([
            (
                ACCEPT,
                HeaderValue::from_static("application/vnd.github+json"),
            ),
            (
                // IMPORTANT: header name muse be lowercase or else this panics
                HeaderName::from_static("x-github-api-version"),
                HeaderValue::from_static("2022-11-28"),
            ),
        ]))
        .build()
        .expect("invalid")
});

/// Make a request to GitHub's API with smart rate limiting
#[tracing::instrument]
pub async fn request<T>(url: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    tracing::debug!("Making GitHub API request");

    let token = get_token(None, None, GitHubAuthPolicy::PreferUser).await;

    apply_rate_limiting(url, token.is_some()).await?;

    let mut request = CLIENT.get(url);
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    let response = request.send().await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    Ok(response.json().await?)
}

/// Make an authenticated GET request to GitHub's API and deserialize the response.
///
/// Unlike [`request`], this accepts explicit `owner`/`repo` for token resolution,
/// which enables repo-specific installation tokens via Stencila Cloud.
#[tracing::instrument(skip(owner, repo))]
pub(crate) async fn get_json<T>(
    url: &str,
    owner: &str,
    repo: &str,
    policy: GitHubAuthPolicy,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let token = get_token(Some(owner), Some(repo), policy).await;

    apply_rate_limiting(url, token.is_some()).await?;

    let mut req = CLIENT.get(url);
    if let Some(token) = &token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let response = req.send().await?;
    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }
    Ok(response.json().await?)
}

/// Send an authenticated request to GitHub's API and return the raw response.
///
/// Resolves a `Bearer` token for the given `owner`/`repo` context, applies
/// rate limiting, and sends the request. Callers are responsible for
/// checking the response status and deserializing the body.
async fn send_authenticated(
    request: reqwest::RequestBuilder,
    url: &str,
    owner: &str,
    repo: &str,
    policy: GitHubAuthPolicy,
) -> Result<reqwest::Response> {
    let token = get_token(Some(owner), Some(repo), policy)
        .await
        .ok_or_else(|| stencila_codec::eyre::eyre!("GitHub authentication required"))?;

    apply_rate_limiting(url, true).await?;

    Ok(request
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?)
}

/// Make an authenticated POST request to GitHub's API with a JSON body.
///
/// Serializes `body` as JSON, attaches a `Bearer` token resolved for the
/// given `owner`/`repo` context, and returns the deserialized response.
#[tracing::instrument(skip(body, owner, repo))]
pub(crate) async fn post_json<B, T>(
    url: &str,
    body: &B,
    owner: &str,
    repo: &str,
    policy: GitHubAuthPolicy,
) -> Result<T>
where
    B: Serialize,
    T: DeserializeOwned,
{
    let response =
        send_authenticated(CLIENT.post(url).json(body), url, owner, repo, policy).await?;

    if let Err(error) = response.error_for_status_ref() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        bail!("{error} (HTTP {status}): {body_text}");
    }

    Ok(response.json().await?)
}

/// Make an authenticated POST request, returning `Err` with the HTTP status
/// code accessible when the request fails.
///
/// Unlike [`post_json`], this returns a [`PostError`] on non-2xx responses so
/// callers can inspect the status code (e.g. to handle 422 specially).
#[tracing::instrument(skip(body, owner, repo))]
pub(crate) async fn post_json_with_status<B, T>(
    url: &str,
    body: &B,
    owner: &str,
    repo: &str,
    policy: GitHubAuthPolicy,
) -> std::result::Result<T, PostError>
where
    B: Serialize,
    T: DeserializeOwned,
{
    let response = send_authenticated(CLIENT.post(url).json(body), url, owner, repo, policy)
        .await
        .map_err(|e| PostError {
            status: None,
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body_text = response.text().await.unwrap_or_default();
        return Err(PostError {
            status: Some(status),
            message: body_text,
        });
    }

    response.json().await.map_err(|e| PostError {
        status: None,
        message: e.to_string(),
    })
}

/// Make an authenticated PATCH request to GitHub's API with a JSON body.
#[tracing::instrument(skip(body, owner, repo))]
pub(crate) async fn patch_json<B>(
    url: &str,
    body: &B,
    owner: &str,
    repo: &str,
    policy: GitHubAuthPolicy,
) -> Result<()>
where
    B: Serialize,
{
    let response =
        send_authenticated(CLIENT.patch(url).json(body), url, owner, repo, policy).await?;

    if let Err(error) = response.error_for_status_ref() {
        let body_text = response.text().await.unwrap_or_default();
        bail!("{error}: {body_text}");
    }

    Ok(())
}

/// Make an authenticated DELETE request to GitHub's API.
#[tracing::instrument(skip(owner, repo))]
pub(crate) async fn delete(
    url: &str,
    owner: &str,
    repo: &str,
    policy: GitHubAuthPolicy,
) -> Result<()> {
    let response = send_authenticated(CLIENT.delete(url), url, owner, repo, policy).await?;

    if let Err(error) = response.error_for_status_ref() {
        let body_text = response.text().await.unwrap_or_default();
        bail!("{error}: {body_text}");
    }

    Ok(())
}

/// Error from [`post_json_with_status`] that preserves the HTTP status code.
#[derive(Debug)]
pub(crate) struct PostError {
    pub status: Option<u16>,
    pub message: String,
}

impl std::fmt::Display for PostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            Some(code) => write!(f, "HTTP {code}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for PostError {}

/// Build a URL for GitHub API endpoints
pub fn api_url(path: &str) -> String {
    if path.starts_with('/') {
        format!("{API_BASE_URL}{path}")
    } else {
        format!("{API_BASE_URL}/{path}")
    }
}

/// Build a URL for GitHub Search API endpoints
///
/// Minimal necessary encoding of values to produce URLs that are readable Also
/// avoids escaping symbols such as # and = which may be used in code // search
/// queries.     
pub fn search_url(endpoint: &str, query_params: &[(&str, String)]) -> String {
    let mut url = format!("{API_BASE_URL}/search/{endpoint}");

    if !query_params.is_empty() {
        let query_string = query_params
            .iter()
            .map(|(name, value)| {
                let encoded = value.replace(" ", "+").replace("&", "%26");
                [name, "=", &encoded].concat()
            })
            .join("&");

        url.push('?');
        url.push_str(&query_string);
    }

    url
}
