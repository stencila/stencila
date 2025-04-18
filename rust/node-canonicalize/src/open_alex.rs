use std::num::NonZeroU32;

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_ratelimit::{all, RateLimiter as ReqwestRateLimiter};

use codec_text::to_text;
use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    reqwest::Client,
    serde::Deserialize,
    tracing,
};
use schema::{AuthorRoleAuthor, Reference};

use crate::{is_doi, is_orcid};

const API_BASE_URL: &str = "https://api.openalex.org";

// Keep below the default rate limit. Although that is documented to be 10 req/s
// for some reason using values >=5 causes hitting of the limit.
// See https://docs.openalex.org/how-to-use-the-api/rate-limits-and-authentication
const MAX_REQUESTS_PER_SECOND: u32 = 3;

static GOVERNOR: Lazy<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    Lazy::new(|| {
        GovernorRateLimiter::direct(Quota::per_second(
            NonZeroU32::new(MAX_REQUESTS_PER_SECOND).expect("not non-zero"),
        ))
    });

struct RateLimiter;

#[async_trait]
impl ReqwestRateLimiter for RateLimiter {
    async fn acquire_permit(&self) {
        GOVERNOR.until_ready().await;
    }
}

static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    ClientBuilder::new(Client::new())
        .with(all(RateLimiter))
        .build()
});

#[derive(Default, Deserialize)]
#[serde(default, crate = "common::serde")]
struct Work {
    id: String,
    doi: Option<String>,
    authorships: Vec<Authorship>,
}

#[derive(Default, Deserialize)]
#[serde(default, crate = "common::serde")]
struct Authorship {
    author: Author,
    institutions: Vec<Institution>,
}

#[derive(Default, Deserialize)]
#[serde(crate = "common::serde")]
struct Author {
    id: String,
    orcid: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct Institution {
    id: String,
    ror: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct WorksResponse {
    results: Vec<Work>,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct AuthorsResponse {
    results: Vec<Author>,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct InstitutionResponse {
    results: Vec<Institution>,
}

/// Canonicalize a [`Reference`] using OpenAlex.
///
/// This will canonicalize the DOI of the reference as well as the ORCIDs of the authors
/// and the RORs of their affiliations.
///
/// It returns early if none of these require canonicalization. Otherwise it searches
/// for the reference by year and title and uses the OpenAlex response to
/// canonicalize each of these ids.
#[tracing::instrument(skip(reference))]
pub(super) async fn reference(reference: &mut Reference) -> Result<()> {
    if is_doi(&reference.doi)
        && reference
            .authors
            .iter()
            .flatten()
            .all(|author| match author {
                schema::Author::Person(person) => is_orcid(&person.orcid),
                schema::Author::AuthorRole(role) => match &role.author {
                    AuthorRoleAuthor::Person(person) => is_orcid(&person.orcid),
                    _ => true,
                },
                _ => true,
            })
    {
        return Ok(());
    }

    let Some(title) = &reference.title else {
        return Ok(());
    };

    let title = to_text(title);
    if title.is_empty() {
        return Ok(());
    }

    let title = title.replace(",", "");

    let mut filters = vec![format!("title.search:{title}")];
    if let Some(year) = reference.date.as_ref().and_then(|date| date.year()) {
        filters.push(format!("publication_date:{year}"))
    }
    let filters = filters
        .into_iter()
        .map(|filter| ("filter", filter))
        .collect_vec();

    let response = CLIENT
        .get(format!("{API_BASE_URL}/works"))
        .query(&filters)
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    let response: WorksResponse = response.json().await?;

    let Some(work) = response.results.first() else {
        return Ok(());
    };

    if let Some(doi) = &work.doi {
        let doi = doi.trim_start_matches("https://doi.org/");
        reference.doi = Some(doi.to_string());
    } else {
        let id = work.id.trim_start_matches("https://openalex.org/");
        reference.doi = Some(format!("10.0000/openalex.{}", id));
    }

    Ok(())
}

/// Get the ORCID for an author from OpenAlex based on their name
///
/// This function should only be called as a fallback if an ORCID can
/// not be derived from authorship of a [`Reference`]`. That is because it
/// searches only by name and a such does not take advantage of the
/// OpenAlex's disambiguation.
///
/// See https://help.openalex.org/hc/en-us/articles/24347048891543-Author-disambiguation
#[tracing::instrument()]
pub(super) async fn orcid(
    family_names: &Option<Vec<String>>,
    given_names: &Option<Vec<String>>,
) -> Result<Option<String>> {
    let Some(family_names) = &family_names else {
        return Ok(None);
    };
    if family_names.is_empty() {
        return Ok(None);
    }

    let mut name = family_names.join(" ");
    if let Some(given_names) = &given_names {
        name = format!("{} {}", given_names.join(" "), name);
    };

    let response = CLIENT
        .get(format!("{API_BASE_URL}/authors"))
        .query(&[("search", name)])
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    let response: AuthorsResponse = response.json().await?;

    let Some(author) = response.results.first() else {
        return Ok(None);
    };

    // If author has an ORCID, return it (with URL prefix stripped)
    if let Some(orcid) = &author.orcid {
        let orcid = orcid.trim_start_matches("https://orcid.org/").into();
        return Ok(Some(orcid));
    }

    // Generate a pseudo-ORCID from the OpenAlex ID
    // Uses 'O' as the first letter to indicate that it is a pseudo-ORCID based on OpenAlex ID
    // (and which OpenAlex author IDs have anyway)
    let int: u64 = author
        .id
        .trim_start_matches("https://openalex.org/")
        .trim_start_matches("A")
        .parse()?;
    let digits = format!("{:015}", int % 1_000_000_000_000_000);
    let pseudo_orcid = format!(
        "O{}-{}-{}-{}",
        &digits[0..3],
        &digits[3..7],
        &digits[7..11],
        &digits[11..15],
    );

    Ok(Some(pseudo_orcid))
}

/// Get the ROR for an organization from OpenAlex based on it's name
///
/// This function should only be called as a fallback if an ROR can
/// not be derived from authorship of a [`Reference`]`.
#[tracing::instrument()]
pub(super) async fn ror(name: &Option<String>) -> Result<Option<String>> {
    let Some(name) = &name else {
        return Ok(None);
    };
    if name.is_empty() {
        return Ok(None);
    }

    let response = CLIENT
        .get(format!("{API_BASE_URL}/institutions"))
        .query(&[("search", name)])
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!("{error}: {}", response.text().await.unwrap_or_default());
    }

    let response: InstitutionResponse = response.json().await?;

    let Some(author) = response.results.first() else {
        return Ok(None);
    };

    // If author has an ROR, return it (with URL prefix stripped)
    if let Some(ror) = &author.ror {
        let ror = ror.trim_start_matches("https://ror.org/").into();
        return Ok(Some(ror));
    }

    // Generate a pseudo-ROR from the OpenAlex ID
    // Uses 'O' as the first letter to indicate that it is a pseudo-ROR based on OpenAlex ID
    let digits = author
        .id
        .trim_start_matches("https://openalex.org/")
        .trim_start_matches('I');
    let pseudo_ror = format!("O{digits}");

    Ok(Some(pseudo_ror))
}
