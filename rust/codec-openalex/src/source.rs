use serde::Deserialize;

use stencila_codec::stencila_schema::{
    CreativeWorkType, Node, Organization, Periodical, PeriodicalOptions, PersonOrOrganization,
    Primitive, PropertyValue, PropertyValueOrString, Reference, ReferenceOptions, shortcuts::t,
};

use crate::ids::{Ids, ids_to_identifiers};

/// An OpenAlex `Source` object
///
/// See https://docs.openalex.org/api-entities/sources/source-object
///
/// Fields not currently used are commented out to reduce bloat and avoid risk
/// or deserialization errors.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Source {
    pub id: String,
    pub display_name: Option<String>,
    pub alternate_titles: Option<Vec<String>>,
    pub abbreviated_title: Option<String>,
    //pub r#type: Option<String>,
    pub homepage_url: Option<String>,
    //pub country_code: Option<String>,
    //pub is_oa: Option<bool>,
    //pub is_in_doaj: Option<bool>,
    //pub is_core: Option<bool>,
    pub host_organization: Option<String>,
    pub host_organization_name: Option<String>,
    //pub host_organization_lineage: Option<Vec<String>>,
    pub issn_l: Option<String>,
    pub issn: Option<Vec<String>>,
    //pub works_count: Option<i64>,
    //pub cited_by_count: Option<i64>,
    //pub summary_stats: Option<SummaryStats>,
    pub ids: Option<Ids>,
    //pub counts_by_year: Option<Vec<CountsByYear>>,
    //pub updated_date: Option<String>,
    //pub created_date: Option<String>,
    //pub societies: Option<Vec<Society>>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedSource {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub issn_l: Option<String>,
    //pub issn: Option<Vec<String>>,
    //pub is_oa: Option<bool>,
    //pub is_in_doaj: Option<bool>,
    //pub is_core: Option<bool>,
    //pub host_organization: Option<String>,
    //pub host_organization_name: Option<String>,
    //pub host_organization_lineage: Option<Vec<String>>,
    pub r#type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct Society {
    pub url: Option<String>,
    pub organization: Option<String>,
}

impl From<Source> for Periodical {
    fn from(source: Source) -> Self {
        let name = source.display_name;

        let mut alternate_names = source.alternate_titles.unwrap_or_default();
        if let Some(abbreviated) = source.abbreviated_title
            && !alternate_names.contains(&abbreviated)
        {
            alternate_names.push(abbreviated);
        }
        let alternate_names = (!alternate_names.is_empty()).then_some(alternate_names);

        let url = source.homepage_url;

        let publisher =
            if source.host_organization.is_some() || source.host_organization_name.is_some() {
                Some(PersonOrOrganization::Organization(Organization {
                    id: source.host_organization,
                    name: source.host_organization_name,
                    ..Default::default()
                }))
            } else {
                None
            };

        let mut issns = Vec::new();
        if let Some(issn_l) = source.issn_l {
            issns.push(issn_l);
        }
        if let Some(source_issns) = source.issn {
            issns.extend(source_issns);
        }
        let issns = (!issns.is_empty()).then_some(issns);

        let identifiers = source.ids.and_then(ids_to_identifiers);

        Periodical {
            name,
            options: Box::new(PeriodicalOptions {
                alternate_names,
                url,
                publisher,
                issns,
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Source> for Node {
    fn from(source: Source) -> Self {
        Node::Periodical(source.into())
    }
}

impl From<&DehydratedSource> for Reference {
    fn from(source: &DehydratedSource) -> Self {
        let work_type = creative_work_type(source.r#type.as_deref());

        let title = source.display_name.as_ref().map(|name| vec![t(name)]);

        let identifiers = if let Some(issn_l) = &source.issn_l {
            Some(vec![PropertyValueOrString::PropertyValue(PropertyValue {
                property_id: Some("issn".into()),
                value: Primitive::String(issn_l.into()),
                ..Default::default()
            })])
        } else {
            None
        };

        Reference {
            work_type,
            title,
            options: Box::new(ReferenceOptions {
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/// Map OpenAlex source type string to CreativeWorkType enum
fn creative_work_type(source_type: Option<&str>) -> Option<CreativeWorkType> {
    source_type.and_then(|source_type| match source_type {
        "journal" => Some(CreativeWorkType::Periodical),
        "repository" => Some(CreativeWorkType::Collection),
        "conference" => Some(CreativeWorkType::Collection),
        "ebook platform" | "book series" | "metadata" | "other" | _ => None,
    })
}
