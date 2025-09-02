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
use serde::de::DeserializeOwned;
use tokio::time::Instant;

use stencila_codec::eyre::{Result, bail};
use stencila_version::STENCILA_USER_AGENT;

const API_BASE_URL: &str = "https://api.github.com";

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
async fn apply_rate_limiting(url: &str, authenticated: bool) -> Result<()> {
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

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
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

    let token = stencila_secrets::env_or_get("GITHUB_TOKEN").ok();

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
