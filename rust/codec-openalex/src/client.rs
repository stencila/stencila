use std::{num::NonZeroU32, sync::LazyLock};

use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use itertools::Itertools;
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use tokio::time::{Duration, Instant, sleep};

use stencila_codec::eyre::{Result, bail};
use stencila_version::STENCILA_USER_AGENT;

use crate::{
    author::Author,
    institution::Institution,
    responses::{AuthorsResponse, IdResponse, InstitutionsResponse, WorksResponse},
    utils::strip_openalex_prefix,
    work::Work,
};

const API_BASE_URL: &str = "https://api.openalex.org";

// OpenAlex rate limits for unauthenticated users
// https://docs.openalex.org/how-to-use-the-api/rate-limits-and-authentication
// 10 requests/second, 100,000 requests/day

// Per-second rate limiter (9 req/s for safety)
static SECOND_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        GovernorRateLimiter::direct(Quota::per_second(NonZeroU32::new(9).expect("invalid")))
    });

// Daily rate limiter approximation (95,000 req/day ≈ 3958 req/hour)
static DAILY_GOVERNOR: LazyLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    LazyLock::new(|| {
        GovernorRateLimiter::direct(Quota::per_hour(NonZeroU32::new(3958).expect("invalid")))
    });

/// Apply rate limiting for OpenAlex API
async fn apply_rate_limiting() -> Result<()> {
    let start = Instant::now();

    SECOND_GOVERNOR.until_ready().await;
    DAILY_GOVERNOR.until_ready().await;

    let duration = (Instant::now() - start).as_millis();
    if duration > 0 {
        tracing::trace!("Rate limited for {duration}ms");
    }

    Ok(())
}

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(STENCILA_USER_AGENT)
        .build()
        .expect("invalid")
});

/// Generate an OpenAlex API URL
///
/// This function, via `request` function below`, may receive URLs with or
/// without the API_BASE_URL. This can not be made consistent (ie. one or other)
/// because the public function `request_list` intentionally uses a full URL
/// (because we want to be able to show that to the user).
///
/// Minimal necessary encoding of values to produce URLs that are readable and
/// similar to those in the OpenAlex docs (e.g. : is not encoded)    
pub fn build_url(path: &str, query_params: &[(&str, String)]) -> String {
    let mut url = if !path.starts_with(API_BASE_URL) {
        [API_BASE_URL, "/", path].concat()
    } else {
        path.to_string()
    };

    if !query_params.is_empty() {
        let query_string = query_params
            .iter()
            .map(|(name, value)| {
                let encoded = value
                    .replace(" ", "+")
                    .replace("?", "%3F")
                    .replace("&", "%26")
                    .replace("=", "%3D")
                    .replace("#", "%23")
                    .replace("%", "%25");
                [name, "=", &encoded].concat()
            })
            .join("&");

        url.push(if path.contains('?') { '&' } else { '?' });
        url.push_str(&query_string);
    }

    url
}

/// Make a a request with retry logic for 429 errors
///
/// This is necessary because 429 (too many requests) can still occur despite
/// governors adhering to documented rate limits.
async fn request(path: &str, query_params: &[(&str, String)]) -> Result<Response> {
    let url = build_url(path, query_params);

    const MAX_RETRIES: u32 = 5;
    for attempt in 0..=MAX_RETRIES {
        apply_rate_limiting().await?;

        let response = CLIENT.get(&url).send().await?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
            let delay_ms = 500 * (1 << attempt); // 500ms, 1000ms, 2000ms etc
            tracing::trace!(
                "Got 429, retrying after {delay_ms}ms (attempt {}/{MAX_RETRIES})",
                attempt + 1
            );
            sleep(Duration::from_millis(delay_ms)).await;
            continue;
        }

        if let Err(error) = response.error_for_status_ref() {
            bail!("{error}: {}", response.text().await.unwrap_or_default());
        }

        return Ok(response);
    }

    unreachable!()
}

/// Generate an OpenAlex API URL to query a list of an entity type             
pub fn list_url(entity_type: &str, query_params: &[(&str, String)]) -> String {
    build_url(entity_type, query_params)
}

/// Make a generic request to the OpenAlex API
///
/// Returns the response as a List<T> where T is the expected entity type
#[tracing::instrument]
pub async fn request_list<T>(url: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    tracing::trace!("Requesting list of entities");

    let url = if !url.contains("&per-page=") {
        // If no per-page specified use the maximum, In the future, more sophisticated
        // page or cursor handling may be implemented
        // See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities/paging
        [url, "&per-page=200"].concat()
    } else {
        url.to_string()
    };

    let response = request(&url, &[]).await?;
    Ok(response.json().await?)
}

/// Make a request for the ids of entities of a type matching filters
#[tracing::instrument(skip(url))]
pub async fn request_ids(url: &str) -> Result<Vec<String>> {
    tracing::trace!("Requesting list of ids");

    let response = request_list::<IdResponse>(url).await?;

    let ids = response
        .results
        .into_iter()
        .map(|item| {
            item.id
                .trim_start_matches("https://openalex.org/")
                .to_string()
        })
        .collect();

    Ok(ids)
}

/// Fetch a work from OpenAlex by DOI
#[tracing::instrument]
pub async fn work_by_doi(doi: &str) -> Result<Option<Work>> {
    tracing::trace!("Fetching work by DOI: {doi}");

    match request(&format!("works/{doi}"), &[]).await {
        Ok(response) => Ok(response.json().await?),
        Err(error) => {
            if error.to_string().to_lowercase().contains("404 not found") {
                return Ok(None);
            } else {
                bail!(error)
            }
        }
    }
}

/// Fetch the references of an OpenAlex work
#[tracing::instrument(skip(work))]
pub async fn fetch_work_references(work: &mut Work) -> Result<()> {
    tracing::trace!("Fetching references of work");

    let id = strip_openalex_prefix(&work.id);

    let response = request("works", &[("filter", format!("cited_by:{id}"))]).await?;
    let response: WorksResponse = response.json().await?;

    work.referenced_works_fetched = response.results;

    Ok(())
}

/// Search for works by title and optional year
#[tracing::instrument]
pub async fn search_works_title_year(title: &str, year: Option<i32>) -> Result<Vec<Work>> {
    tracing::trace!("Searching works by title: {title}");

    let title = title.replace(",", " ");

    let mut filters = vec![format!("title.search:{title}")];
    if let Some(year) = year {
        filters.push(format!("publication_date:{year}"))
    }
    let filters = filters
        .into_iter()
        .map(|filter| ("filter", filter))
        .collect_vec();

    let response = request("works", &filters).await?;
    let response: WorksResponse = response.json().await?;
    Ok(response.results)
}

/// Search for works by general search
#[tracing::instrument]
pub async fn search_works(text: &str) -> Result<Vec<Work>> {
    tracing::trace!("Searching works: {text}");

    let response = request("works", &[("search", text.into())]).await?;
    let response: WorksResponse = response.json().await?;
    Ok(response.results)
}

/// Search for authors by name
#[tracing::instrument]
pub async fn search_authors(name: &str) -> Result<Vec<Author>> {
    tracing::trace!("Searching authors: {name}");

    let response = request("authors", &[("search", name.into())]).await?;
    let response: AuthorsResponse = response.json().await?;
    Ok(response.results)
}

/// Search for institutions by name
#[tracing::instrument]
pub async fn search_institutions(name: &str) -> Result<Vec<Institution>> {
    tracing::trace!("Searching institutions: {name}");

    let response = request("institutions", &[("search", name.into())]).await?;
    let response: InstitutionsResponse = response.json().await?;
    Ok(response.results)
}
