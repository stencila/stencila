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
    eyre::{bail, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
    reqwest::Client,
    serde::Deserialize,
    tokio::time::Instant,
    tracing,
};
use schema::{AuthorRole, AuthorRoleAuthor, Reference};

use crate::{is_doi, is_orcid, is_ror};

/// Is an optional id a pseudo ORCID based on from OpenAlex authorship (not name match)
pub(super) fn is_authorship_orcid(id: &Option<String>) -> bool {
    let Some(id) = id else { return false };
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^A000-\d{4}-\d{4}-\d{4}$").expect("invalid regex"));
    REGEX.is_match(id)
}

/// Is an optional id a pseudo ROR based on from OpenAlex authorship
pub(super) fn is_authorship_ror(id: &Option<String>) -> bool {
    let Some(id) = id else { return false };
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^A\d+$").expect("invalid regex"));
    REGEX.is_match(id)
}

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

#[derive(Clone, Default, Deserialize)]
#[serde(default, crate = "common::serde")]
struct Work {
    id: String,
    doi: Option<String>,
    authorships: Vec<Authorship>,
}

impl Work {
    /// Get the DOI of a work, or generate a pseudo DOI
    fn doi(&self) -> String {
        if let Some(doi) = &self.doi {
            doi.trim_start_matches("https://doi.org/").into()
        } else {
            let id = self.id.trim_start_matches("https://openalex.org/");
            format!("10.0000/openalex.{}", id)
        }
    }
}

#[derive(Clone, Default, Deserialize)]
#[serde(default, crate = "common::serde")]
struct Authorship {
    author: Author,
    institutions: Vec<Institution>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default, crate = "common::serde")]
struct Author {
    id: String,
    orcid: Option<String>,
    display_name: String,
}

impl Author {
    /// Get the ORCID of an author, or generate a pseudo ORCID
    fn orcid(&self, prefix: char) -> Result<String> {
        if let Some(orcid) = &self.orcid {
            return Ok(orcid.trim_start_matches("https://orcid.org/").into());
        }

        // Generate a pseudo-ORCID from the OpenAlex ID
        // Uses 'O' as the first letter to indicate that it is a pseudo-ORCID based on OpenAlex ID
        // (and which OpenAlex author IDs have anyway)
        let int: u64 = self
            .id
            .trim_start_matches("https://openalex.org/")
            .trim_start_matches("A")
            .parse()?;
        let digits = format!("{:015}", int % 1_000_000_000_000_000);
        Ok(format!(
            "{prefix}{}-{}-{}-{}",
            &digits[0..3],
            &digits[3..7],
            &digits[7..11],
            &digits[11..15],
        ))
    }
}

#[derive(Clone, Default, Deserialize)]
#[serde(default, crate = "common::serde")]
struct Institution {
    id: String,
    ror: Option<String>,
    display_name: String,
}

impl Institution {
    /// Get the ROR of an institution, or generate a pseudo ROR
    fn ror(&self, prefix: char) -> String {
        if let Some(ror) = &self.ror {
            ror.trim_start_matches("https://ror.org/").into()
        } else {
            // Generate a pseudo-ROR from the OpenAlex ID
            // Uses 'O' as the first letter to indicate that it is a pseudo-ROR based on OpenAlex ID
            let digits = self
                .id
                .trim_start_matches("https://openalex.org/")
                .trim_start_matches('I');
            format!("{prefix}{digits}")
        }
    }
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
#[tracing::instrument(skip(reference))]
pub(super) async fn reference(reference: &mut Reference) -> Result<()> {
    let work: Work = if let (Some(doi), true) = (&reference.doi, is_doi(&reference.doi)) {
        tracing::trace!("Fetching work");

        let response = CLIENT
            .get(format!("{API_BASE_URL}/works/https://doi.org/{doi}"))
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            bail!("{error}: {}", response.text().await.unwrap_or_default());
        }

        response.json().await?
    } else {
        let Some(title) = &reference.title else {
            return Ok(());
        };

        let title = to_text(title);
        if title.is_empty() {
            return Ok(());
        }

        let title = title.replace(",", " ");

        let mut filters = vec![format!("title.search:{title}")];
        if let Some(year) = reference.date.as_ref().and_then(|date| date.year()) {
            filters.push(format!("publication_date:{year}"))
        }
        let filters = filters
            .into_iter()
            .map(|filter| ("filter", filter))
            .collect_vec();

        tracing::trace!("Searching for work");

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

        work.clone()
    };

    // Canonicalize DOI if necessary
    if !is_doi(&reference.doi) {
        reference.doi = Some(work.doi());
    }

    // Canonicalize the ORCID's of authors and the ROR's of their affiliations
    // To avoid mis-assignment due to differences in order of authors, for any change to be made
    // the author's first name must be in the `display_name`.
    for (author, authorship) in reference
        .authors
        .iter_mut()
        .flatten()
        .zip(work.authorships.iter())
    {
        if let schema::Author::Person(person)
        | schema::Author::AuthorRole(AuthorRole {
            author: AuthorRoleAuthor::Person(person),
            ..
        }) = author
        {
            let Some(name) = human_name::Name::parse(&person.name()) else {
                continue;
            };
            let Some(oa_name) = human_name::Name::parse(&authorship.author.display_name) else {
                continue;
            };

            if !oa_name.consistent_with(&name) {
                tracing::debug!(
                    "`{}` not consistent with `{}`",
                    name.display_full(),
                    oa_name.display_full()
                );

                continue;
            }

            if !is_orcid(&person.orcid) {
                person.orcid = Some(authorship.author.orcid('A')?);
            }

            for org in person.affiliations.iter_mut().flatten() {
                let Some(name) = &org.name else { continue };
                if !is_ror(&org.ror) {
                    for inst in &authorship.institutions {
                        if name.contains(&inst.display_name) {
                            org.ror = Some(inst.ror('A'));
                            break;
                        }
                    }
                }
            }
        }
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

    tracing::trace!("Searching for author");

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

    Ok(Some(author.orcid('O')?))
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

    let mut name = name.to_string();

    loop {
        tracing::trace!("Searching for institution: {name}");

        let response = CLIENT
            .get(format!("{API_BASE_URL}/institutions"))
            .query(&[("search", &name)])
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            bail!("{error}: {}", response.text().await.unwrap_or_default());
        }

        let response: InstitutionResponse = response.json().await?;

        if let Some(inst) = response.results.first() {
            return Ok(Some(inst.ror('O')));
        }

        // Try successively removing sub-orgs (e.g. departments) from the
        // org until no more commas
        if let Some(comma) = name.find(',') {
            if comma + 1 >= name.len() {
                break;
            }
            name = name[(comma + 1)..].to_string();
        } else {
            break;
        }
    }

    Ok(None)
}
