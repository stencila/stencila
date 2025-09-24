use std::str::FromStr;

use serde::Deserialize;

use stencila_codec::{
    eyre::{Result, bail},
    stencila_schema::{Node, Organization, Person, PersonOptions},
};

use crate::{
    ids::{Ids, id_to_identifiers, ids_to_identifiers},
    institution::DehydratedInstitution,
    utils::{get_or_generate_orcid, strip_orcid_prefix},
};

/// An OpenAlex `Author` object
///
/// See https://docs.openalex.org/api-entities/authors/author-object
///
/// Fields not currently used are commented out to reduce bloat and avoid risk
/// or deserialization errors.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Author {
    pub id: String,
    pub orcid: Option<String>,
    pub display_name: Option<String>,
    pub display_name_alternatives: Option<Vec<String>>,
    pub ids: Option<Ids>,
    //pub works_count: Option<i64>,
    //pub cited_by_count: Option<i64>,
    //pub summary_stats: Option<SummaryStats>,
    pub affiliations: Option<Vec<Affiliation>>,
    pub last_known_institutions: Option<Vec<DehydratedInstitution>>,
    //pub works_api_url: Option<String>,
    //pub updated_date: Option<String>,
    //pub created_date: Option<String>,
    //pub counts_by_year: Option<Vec<CountsByYear>>,
    //pub x_concepts: Option<Vec<Concept>>,
}

impl Author {
    /// Get the ORCID of an author, or generate a pseudo ORCID
    pub fn orcid(&self, prefix: char) -> Result<String> {
        get_or_generate_orcid(&self.orcid, &self.id, prefix)
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedAuthor {
    pub id: Option<String>,
    pub orcid: Option<String>,
    pub display_name: Option<String>,
}

impl DehydratedAuthor {
    /// Get the ORCID of an author, or generate a pseudo ORCID
    pub fn orcid(&self, prefix: char) -> Result<String> {
        if let Some(id) = &self.id {
            get_or_generate_orcid(&self.orcid, id, prefix)
        } else {
            bail!("Missing author ID")
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Affiliation {
    pub institution: Option<DehydratedInstitution>,
    pub years: Option<Vec<i32>>,
}

impl From<Author> for Person {
    fn from(author: Author) -> Self {
        let orcid = strip_orcid_prefix(author.orcid);

        let (name, parsed) = create_person_from_display_name(author.display_name.clone());

        // Map display_name_alternatives to alternate_names, avoiding duplicates with display_name
        let alternate_names = author.display_name_alternatives.and_then(|alternatives| {
            let mut filtered_names = Vec::new();
            for name in alternatives {
                // Normalize both names for comparison by replacing various dash characters with regular hyphen
                let normalized_name = name.replace(['‐', '–', '—'], "-");

                let normalized_display_name = author
                    .display_name
                    .as_ref()
                    .map(|n| n.replace(['‐', '–', '—'], "-"));

                // Only include if it's different from the main display_name after normalization
                if normalized_display_name.as_ref() != Some(&normalized_name) {
                    filtered_names.push(name);
                }
            }
            (!filtered_names.is_empty()).then_some(filtered_names)
        });

        // Map affiliations from affiliations or last_known_institutions
        let affiliations = if let Some(affiliations) = author.affiliations {
            let organizations: Vec<Organization> = affiliations
                .into_iter()
                .filter_map(|affiliation| affiliation.institution)
                .map(Organization::from)
                .collect();
            (!organizations.is_empty()).then_some(organizations)
        } else if let Some(last_known) = author.last_known_institutions {
            let organizations: Vec<Organization> =
                last_known.into_iter().map(Organization::from).collect();
            (!organizations.is_empty()).then_some(organizations)
        } else {
            None
        };

        let identifiers = author.ids.and_then(ids_to_identifiers);

        Person {
            orcid,
            affiliations,
            options: Box::new(PersonOptions {
                name,
                alternate_names,
                identifiers,
                ..Default::default()
            }),
            // Parsed names may include given names, family name, honorifics etc
            ..parsed.unwrap_or_default()
        }
    }
}

impl From<Author> for Node {
    fn from(author: Author) -> Self {
        Node::Person(author.into())
    }
}

impl From<DehydratedAuthor> for Person {
    fn from(author: DehydratedAuthor) -> Self {
        let orcid = strip_orcid_prefix(author.orcid);

        let (name, parsed) = create_person_from_display_name(author.display_name);

        let identifiers = author.id.and_then(|id| id_to_identifiers("openalex", id));

        Person {
            orcid,
            options: Box::new(PersonOptions {
                name,
                identifiers,
                ..Default::default()
            }),
            // Parsed names may include given names, family name, honorifics etc
            ..parsed.unwrap_or_default()
        }
    }
}

/// Helper function to create a Person from basic author information
fn create_person_from_display_name(
    display_name: Option<String>,
) -> (Option<String>, Option<Person>) {
    let name = display_name.clone();

    // Try to parse given_names and family_names from display_name
    let parsed = if let Some(name) = &name {
        Person::from_str(name).ok()
    } else {
        None
    };

    (name, parsed)
}
