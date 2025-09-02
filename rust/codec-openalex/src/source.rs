use serde::Deserialize;

use indexmap::IndexMap;
use stencila_codec::stencila_schema::{
    Node, Organization, Periodical, PeriodicalOptions, PersonOrOrganization,
};

use crate::utils::convert_ids_to_identifiers;

/// An OpenAlex `Source` object
///
/// See https://docs.openalex.org/api-entities/sources/source-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Source {
    pub id: String,
    pub display_name: Option<String>,
    pub alternate_titles: Option<Vec<String>>,
    pub abbreviated_title: Option<String>,
    pub r#type: Option<String>,
    pub homepage_url: Option<String>,
    pub country_code: Option<String>,
    pub is_oa: Option<bool>,
    pub is_in_doaj: Option<bool>,
    pub is_core: Option<bool>,
    pub host_organization: Option<String>,
    pub host_organization_name: Option<String>,
    pub host_organization_lineage: Option<Vec<String>>,
    pub issn_l: Option<String>,
    pub issn: Option<Vec<String>>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub summary_stats: Option<SummaryStats>,
    pub ids: Option<ExternalIds>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,
    pub societies: Option<Vec<Society>>,
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
    pub issn_l: Option<String>,
    pub issn: Option<Vec<String>>,
    pub wikidata: Option<String>,
    pub fatcat: Option<String>,
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
pub struct Society {
    pub url: Option<String>,
    pub organization: Option<String>,
}

/// Convert Source ExternalIds to IndexMap for use with convert_ids_to_identifiers
///
/// Note: does not include ISSNs since there is a specific property for those.
fn convert_source_ids_to_indexmap(ids: &ExternalIds) -> IndexMap<String, String> {
    let mut id_map = IndexMap::new();

    if let Some(openalex) = &ids.openalex {
        id_map.insert("openalex".to_string(), openalex.clone());
    }

    if let Some(wikidata) = &ids.wikidata {
        id_map.insert("wikidata".to_string(), wikidata.clone());
    }

    if let Some(fatcat) = &ids.fatcat {
        id_map.insert("fatcat".to_string(), fatcat.clone());
    }

    id_map
}

impl From<Source> for Periodical {
    fn from(source: Source) -> Self {
        // Map alternative titles and abbreviated titles
        let mut alternate_names = source.alternate_titles.unwrap_or_default();
        if let Some(abbreviated) = source.abbreviated_title
            && !alternate_names.contains(&abbreviated)
        {
            alternate_names.push(abbreviated);
        }
        let alternate_names = (!alternate_names.is_empty()).then_some(alternate_names);

        // Map publisher (host organization)
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

        // Map ISSN information
        let mut issns = Vec::new();
        if let Some(issn_l) = source.issn_l {
            issns.push(issn_l);
        }
        if let Some(source_issns) = source.issn {
            issns.extend(source_issns);
        }
        let issns = (!issns.is_empty()).then_some(issns);

        // Map other ids to identifiers
        let identifiers = source.ids.as_ref().and_then(|ids| {
            let id_map = convert_source_ids_to_indexmap(ids);
            convert_ids_to_identifiers(&id_map)
        });

        Periodical {
            id: Some(source.id),
            name: source.display_name,
            options: Box::new(PeriodicalOptions {
                url: source.homepage_url,
                alternate_names,
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
