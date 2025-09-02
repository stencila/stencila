use std::{num::NonZeroU32, sync::LazyLock};

use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use itertools::Itertools;
use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use tokio::time::Instant;

use stencila_codec::eyre::{Result, bail};
use stencila_version::STENCILA_USER_AGENT;

const API_BASE_URL: &str = "https://zenodo.org/api";

// Rate limits based on Zenodo API documentation
// https://developers.zenodo.org/#rate-limiting

// Guest API rate limiter
static GUEST_MINUTE_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 60 requests/minute => 58 requests/minute for safety
        GovernorRateLimiter::direct(Quota::per_minute(NonZeroU32::new(58).expect("invalid")))
    });

// Authenticated API rate limiter
static AUTH_MINUTE_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 100 requests/minute => 95 requests/minute for safety
        GovernorRateLimiter::direct(Quota::per_minute(NonZeroU32::new(95).expect("invalid")))
    });

// Guest hourly rate limiter
static GUEST_HOURLY_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 2000 requests/hour => 1950 requests/hour for safety
        GovernorRateLimiter::direct(Quota::per_hour(NonZeroU32::new(1950).expect("invalid")))
    });

// Authenticated hourly rate limiter
static AUTH_HOURLY_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        // 5000 requests/hour => 4900 requests/hour for safety
        GovernorRateLimiter::direct(Quota::per_hour(NonZeroU32::new(4900).expect("invalid")))
    });

/// Apply rate limiting based on authentication status
async fn apply_rate_limiting(authenticated: bool) -> Result<()> {
    let start = Instant::now();

    if authenticated {
        AUTH_MINUTE_GOVERNOR.until_ready().await;
        AUTH_HOURLY_GOVERNOR.until_ready().await;
    } else {
        GUEST_MINUTE_GOVERNOR.until_ready().await;
        GUEST_HOURLY_GOVERNOR.until_ready().await;
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
        .default_headers(HeaderMap::from_iter([(
            ACCEPT,
            HeaderValue::from_static("application/json"),
        )]))
        .build()
        .expect("invalid")
});

/// Make a request to Zenodo's API with rate limiting
#[tracing::instrument]
pub async fn request<T>(url: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    tracing::debug!("Making Zenodo API request");

    let token = stencila_secrets::env_or_get("ZENODO_TOKEN").ok();

    apply_rate_limiting(token.is_some()).await?;

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

/// Build a URL for Zenodo API endpoints
pub fn api_url(path: &str) -> String {
    if path.starts_with('/') {
        format!("{API_BASE_URL}{path}")
    } else {
        format!("{API_BASE_URL}/{path}")
    }
}

/// Build a URL for Zenodo Search API endpoints
pub fn search_url(query_params: &[(&str, String)]) -> String {
    let mut url = format!("{API_BASE_URL}/records");

    if !query_params.is_empty() {
        let query_string = query_params
            .iter()
            .map(|(name, value)| {
                let encoded = value.replace(" ", "+").replace("&", "%26");
                format!("{name}={encoded}")
            })
            .join("&");

        url.push('?');
        url.push_str(&query_string);
    }

    url
}
