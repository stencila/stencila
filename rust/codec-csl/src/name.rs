use std::str::FromStr;

use codec::{common::{
    indexmap::IndexMap, serde::{Deserialize, Serialize}, serde_json::Value, serde_with::skip_serializing_none
}, schema::{Author, Person}};

/// A CSL name field
/// 
/// Represents contributor names in CSL-JSON format, supporting both personal names
/// with separate components and literal names for institutions or single-name persons.
/// 
/// See:
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#name-fields
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", crate = "codec::common::serde")]
pub struct NameField {
    /// Family/surname
    pub family: Option<String>,

    /// Given/first name(s)
    pub given: Option<String>,

    /// Complete name for institutions or single-name persons
    pub literal: Option<String>,

    /// Name particles that are dropped in certain contexts (e.g., "Rev.")
    pub dropping_particle: Option<String>,

    /// Name particles that are not dropped (e.g., "de", "van")
    pub non_dropping_particle: Option<String>,

    /// Name suffix (e.g., "Jr.", "Ph.D.")
    pub suffix: Option<String>,

    /// Ordering of the contributor (e.g., "first", "additional")
    pub sequence: Option<String>,

    /// Institutional affiliations
    pub affiliation: Option<Vec<Value>>,

    /// ORCID identifier
    pub orcid: Option<String>,
    
    /// Additional name-related fields
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}


/// Convert CSL authors to Stencila authors
pub fn convert_csl_authors(csl_authors: &[NameField]) -> Vec<Author> {
    csl_authors
        .iter()
        .filter_map(|csl_author| {
            if let Some(literal) = &csl_author.literal {
                // Try to parse the literal name
                match Person::from_str(literal) {
                    Ok(person) => Some(Author::Person(person)),
                    Err(_) => {
                        // Fall back to literal name as given name
                        Some(Author::Person(Person {
                            given_names: Some(vec![literal.clone()]),
                            ..Default::default()
                        }))
                    }
                }
            } else {
                // Build from parts
                let mut person = Person::default();
                if let Some(given) = &csl_author.given {
                    person.given_names = Some(vec![given.clone()]);
                }
                if let Some(family) = &csl_author.family {
                    person.family_names = Some(vec![family.clone()]);
                }

                // Only create a person if we have at least some name information
                if person.given_names.is_some() || person.family_names.is_some() {
                    Some(Author::Person(person))
                } else {
                    None
                }
            }
        })
        .collect()
}
