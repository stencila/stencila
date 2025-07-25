use std::num::NonZeroU32;

use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_ratelimit::{RateLimiter as ReqwestRateLimiter, all};

use codec::common::{
    eyre::{Result, bail},
    itertools::Itertools,
    once_cell::sync::Lazy,
    reqwest::Client,
    serde::de::DeserializeOwned,
    tokio::time::Instant,
    tracing,
};

use crate::{
    author::Author,
    institution::Institution,
    responses::{AuthorsResponse, InstitutionsResponse, WorksResponse},
    work::Work,
};

const API_BASE_URL: &str = "https://api.openalex.org";

// Keep below the default rate limit. Although that is documented to be 10 req/s
// for some reason using values >=4 causes hitting of the limit.
// See https://docs.openalex.org/how-to-use-the-api/rate-limits-and-authentication
const MAX_REQUESTS_PER_SECOND: u32 = 3;

static GOVERNOR: Lazy<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    Lazy::new(|| {
        GovernorRateLimiter::direct(Quota::per_second(
            NonZeroU32::new(MAX_REQUESTS_PER_SECOND).expect("not non-zero"),
        ))
    });

struct RateLimiter;

impl ReqwestRateLimiter for RateLimiter {
    async fn acquire_permit(&self) {
        let start = Instant::now();
        GOVERNOR.until_ready().await;
        tracing::trace!(
            "Rate limited for {}ms",
            (Instant::now() - start).as_millis()
        )
    }
}

static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    ClientBuilder::new(Client::new())
        .with(all(RateLimiter))
        .build()
});

/// Make a generic request to the OpenAlex API with custom query parameters
/// Returns the response as a List<T> where T is the expected entity type
pub async fn request_with_params<T>(entity_type: &str, params: &[(&str, String)]) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut url = format!("{API_BASE_URL}/{entity_type}");

    if !params.is_empty() {
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&query_string);
    }

    tracing::trace!("Making OpenAlex API request with params: {}", url);

    let response = CLIENT.get(&url).send().await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    Ok(response.json().await?)
}

/// Fetch a work from OpenAlex by DOI
#[tracing::instrument()]
pub async fn work_by_doi(doi: &str) -> Result<Work> {
    tracing::trace!("Fetching work by DOI: {}", doi);

    let response = CLIENT
        .get(format!("{API_BASE_URL}/works/https://doi.org/{doi}"))
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    Ok(response.json().await?)
}

/// Search for works by title and optional year
pub async fn search_works(title: &str, year: Option<i32>) -> Result<Vec<Work>> {
    let title = title.replace(",", " ");

    let mut filters = vec![format!("title.search:{title}")];
    if let Some(year) = year {
        filters.push(format!("publication_date:{year}"))
    }
    let filters = filters
        .into_iter()
        .map(|filter| ("filter", filter))
        .collect_vec();

    tracing::trace!("Searching for work with title: {}", title);

    let response = CLIENT
        .get(format!("{API_BASE_URL}/works"))
        .query(&filters)
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    let response: WorksResponse = response.json().await?;
    Ok(response.results)
}

/// Search for authors by name
pub async fn search_authors(name: &str) -> Result<Vec<Author>> {
    tracing::trace!("Searching for author: {}", name);

    let response = CLIENT
        .get(format!("{API_BASE_URL}/authors"))
        .query(&[("search", name)])
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    let response: AuthorsResponse = response.json().await?;
    Ok(response.results)
}

/// Search for institutions by name
pub async fn search_institutions(name: &str) -> Result<Vec<Institution>> {
    tracing::trace!("Searching for institution: {}", name);

    let response = CLIENT
        .get(format!("{API_BASE_URL}/institutions"))
        .query(&[("search", name)])
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    let response: InstitutionsResponse = response.json().await?;
    Ok(response.results)
}
