use codec_openalex::{
    search_authors, search_institutions, search_works, work_by_doi, OpenAlexWork,
};
use codec_text::to_text;
use common::{eyre::Result, once_cell::sync::Lazy, regex::Regex, tracing};
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

/// Canonicalize a [`Reference`] using OpenAlex.
///
/// This will canonicalize the DOI of the reference as well as the ORCIDs of the authors
/// and the RORs of their affiliations.
#[tracing::instrument(skip(reference))]
pub(super) async fn reference(reference: &mut Reference) -> Result<()> {
    let work: OpenAlexWork = if let (Some(doi), true) = (&reference.doi, is_doi(&reference.doi)) {
        tracing::trace!("Fetching work");
        work_by_doi(doi).await?
    } else {
        let Some(title) = &reference.title else {
            return Ok(());
        };

        let title = to_text(title);
        if title.is_empty() {
            return Ok(());
        }

        let year = reference
            .date
            .as_ref()
            .and_then(|date| date.year())
            .map(|y| y as i32);
        let works = search_works(&title, year).await?;
        let Some(work) = works.first() else {
            return Ok(());
        };

        (*work).clone()
    };

    // Canonicalize DOI if necessary
    if !is_doi(&reference.doi) {
        reference.doi = Some(work.doi());
    }

    // Canonicalize the ORCID's of authors and the ROR's of their affiliations
    // To avoid mis-assignment due to differences in order of authors, for any change to be made
    // the author's first name must be in the `display_name`.
    if let Some(authorships) = work.authorships {
        for (author, authorship) in reference
            .authors
            .iter_mut()
            .flatten()
            .zip(authorships.iter())
        {
            if let schema::Author::Person(person)
            | schema::Author::AuthorRole(AuthorRole {
                author: AuthorRoleAuthor::Person(person),
                ..
            }) = author
            {
                let Some(authorship_author) = &authorship.author else {
                    continue;
                };
                let Some(authorship_display_name) = &authorship_author.display_name else {
                    continue;
                };

                let Some(name) = human_name::Name::parse(&person.name()) else {
                    continue;
                };
                let Some(oa_name) = human_name::Name::parse(authorship_display_name) else {
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
                    person.orcid = Some(authorship_author.orcid('A')?);
                }

                for org in person.affiliations.iter_mut().flatten() {
                    let Some(name) = &org.name else { continue };
                    if !is_ror(&org.ror) {
                        if let Some(institutions) = &authorship.institutions {
                            for inst in institutions {
                                if let Some(inst_name) = &inst.display_name {
                                    if name.contains(inst_name) {
                                        org.ror = Some(inst.ror('A'));
                                        break;
                                    }
                                }
                            }
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

    let authors = search_authors(&name).await?;

    let Some(author) = authors.first() else {
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
        let institutions = search_institutions(&name).await?;

        if let Some(inst) = institutions.first() {
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
