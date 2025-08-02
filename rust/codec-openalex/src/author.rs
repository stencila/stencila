use std::str::FromStr;

use serde::Deserialize;

use codec::{
    common::eyre::Result,
    schema::{Node, Organization, OrganizationOptions, Person, PersonOptions},
};

/// An OpenAlex `Author` object
///
/// See https://docs.openalex.org/api-entities/authors/author-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Author {
    pub id: String,
    pub orcid: Option<String>,
    pub display_name: Option<String>,
    pub display_name_alternatives: Option<Vec<String>>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub summary_stats: Option<SummaryStats>,
    pub ids: Option<ExternalIds>,
    pub affiliations: Option<Vec<Affiliation>>,
    pub last_known_institutions: Option<Vec<DehydratedInstitution>>,
    pub works_api_url: Option<String>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub x_concepts: Option<Vec<Concept>>,
}

impl Author {
    /// Get the ORCID of an author, or generate a pseudo ORCID
    pub fn orcid(&self, prefix: char) -> Result<String> {
        crate::utils::get_or_generate_orcid(&self.orcid, &self.id, prefix)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SummaryStats {
    #[serde(rename = "2yr_mean_citedness")]
    pub impact_factor: Option<f64>,
    pub h_index: Option<i32>,
    pub i10_index: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExternalIds {
    pub openalex: Option<String>,
    pub orcid: Option<String>,
    pub scopus: Option<String>,
    pub twitter: Option<String>,
    pub wikipedia: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Affiliation {
    pub institution: Option<DehydratedInstitution>,
    pub years: Option<Vec<i32>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedInstitution {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub ror: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub lineage: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Concept {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}

impl From<Author> for Person {
    fn from(author: Author) -> Self {
        // Try to parse given_names and family_names from display_name
        let parsed = if let Some(display_name) = &author.display_name {
            Person::from_str(display_name).ok()
        } else {
            None
        };

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
                .filter_map(|affiliation| {
                    affiliation.institution.map(|inst| Organization {
                        id: inst.id,
                        name: inst.display_name,
                        ror: crate::strip_ror_prefix(inst.ror),
                        options: Box::new(OrganizationOptions::default()),
                        ..Default::default()
                    })
                })
                .collect();

            (!organizations.is_empty()).then_some(organizations)
        } else if let Some(last_known) = author.last_known_institutions {
            let organizations: Vec<Organization> = last_known
                .into_iter()
                .map(|inst| Organization {
                    id: inst.id,
                    name: inst.display_name,
                    ror: crate::strip_ror_prefix(inst.ror),
                    options: Box::new(OrganizationOptions::default()),
                    ..Default::default()
                })
                .collect();

            (!organizations.is_empty()).then_some(organizations)
        } else {
            None
        };

        Person {
            id: Some(author.id),
            orcid: crate::strip_orcid_prefix(author.orcid),
            affiliations,
            options: Box::new(PersonOptions {
                name: author.display_name,
                alternate_names,
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
