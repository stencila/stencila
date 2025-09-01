use std::str::FromStr;

use serde::Deserialize;

use codec::{
    schema::{Author, Organization, Person, PersonOptions},
};

use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::Value;
use serde_with::skip_serializing_none;

/// A CSL name field
///
/// Represents contributor names in CSL-JSON format, supporting both personal names
/// with separate components and literal names for institutions or single-name persons.
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iv-variables (Name Variables)
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#name-fields
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
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

impl From<NameField> for Author {
    fn from(name: NameField) -> Self {
        if name.family.is_none()
            && name.given.is_none()
            && let Some(literal) = name.literal
        {
            let person = Person::from_str(&literal).unwrap_or_else(|_| Person {
                options: Box::new(PersonOptions {
                    name: Some(literal),
                    ..Default::default()
                }),
                ..Default::default()
            });
            return Author::Person(person);
        }

        let family_names = name
            .family
            .map(|name| name.split_whitespace().map(String::from).collect());

        let given_names = name
            .given
            .map(|name| name.split_whitespace().map(String::from).collect());

        let affiliations = name
            .affiliation
            .into_iter()
            .flatten()
            .filter_map(|value| match value {
                Value::String(name) => Some(name),
                Value::Object(object) => object
                    .get("name")
                    .and_then(|value| value.as_str())
                    .map(String::from),
                _ => None,
            })
            .map(|name| Organization {
                name: Some(name),
                ..Default::default()
            })
            .collect_vec();
        let affiliations = (!affiliations.is_empty()).then_some(affiliations);

        Author::Person(Person {
            family_names,
            given_names,
            affiliations,
            orcid: name.orcid,
            options: Box::new(PersonOptions {
                name: name.literal,
                honorific_prefix: name.dropping_particle,
                honorific_suffix: name.suffix,
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
