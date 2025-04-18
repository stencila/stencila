use codec_text::to_text;
use common::{eyre::Result, reqwest, serde::Deserialize};
use schema::{AuthorRoleAuthor, Reference};

use crate::{is_doi, is_orcid};

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

    let mut filters = vec![format!("filter=title.search:{title}")];
    if let Some(year) = reference.date.as_ref().and_then(|date| date.year()) {
        filters.push(format!("filter=publication_date:{year}"))
    }
    let filters = filters.join("&");

    let response: WorksResponse = reqwest::get(format!("https://api.openalex.org/works?{filters}"))
        .await?
        .json()
        .await?;

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

    let response: AuthorsResponse =
        reqwest::get(format!("https://api.openalex.org/authors?search={name}"))
            .await?
            .json()
            .await?;

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
pub(super) async fn ror(name: &Option<String>) -> Result<Option<String>> {
    let Some(name) = &name else {
        return Ok(None);
    };
    if name.is_empty() {
        return Ok(None);
    }

    let response: InstitutionResponse = reqwest::get(format!(
        "https://api.openalex.org/institutions?search={name}"
    ))
    .await?
    .json()
    .await?;

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
