use serde::Deserialize;

use codec::schema::{ImageObject, Node, Organization, OrganizationOptions};
use indexmap::IndexMap;

use crate::utils::convert_ids_to_identifiers;

/// An OpenAlex `Institution` object
///
/// See https://docs.openalex.org/api-entities/institutions/institution-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Institution {
    pub id: String,
    pub ror: Option<String>,
    pub display_name: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub homepage_url: Option<String>,
    pub image_url: Option<String>,
    pub image_thumbnail_url: Option<String>,
    pub display_name_acronyms: Option<Vec<String>>,
    pub display_name_alternatives: Option<Vec<String>>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub summary_stats: Option<SummaryStats>,
    pub ids: Option<ExternalIds>,
    pub geo: Option<Geo>,
    pub international: Option<International>,
    pub associated_institutions: Option<Vec<AssociatedInstitution>>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub works_api_url: Option<String>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,
    pub lineage: Option<Vec<String>>,
    pub roles: Option<Vec<Role>>,
    pub x_concepts: Option<Vec<Concept>>,
}

impl Institution {
    /// Get the ROR of an institution, or generate a pseudo ROR
    pub fn ror(&self, prefix: char) -> String {
        crate::utils::get_or_generate_ror(&self.ror, &self.id, prefix)
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
    pub ror: Option<String>,
    pub grid: Option<String>,
    pub wikipedia: Option<String>,
    pub wikidata: Option<String>,
    pub mag: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Geo {
    pub city: Option<String>,
    pub geonames_city_id: Option<String>,
    pub region: Option<String>,
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct International {
    pub display_name: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssociatedInstitution {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub ror: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub relationship: Option<String>,
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
pub struct Role {
    pub role: Option<String>,
    pub id: Option<String>,
    pub works_count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Concept {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}

impl From<Institution> for Organization {
    fn from(institution: Institution) -> Self {
        // Get ROR
        let ror = crate::strip_ror_prefix(
            institution
                .ror
                .clone()
                .or(institution.ids.as_ref().and_then(|ids| ids.ror.clone())),
        );

        // Map display_name_acronyms and display_name_alternatives to alternate_names
        let mut alternate_names = institution.display_name_alternatives.unwrap_or_default();
        if let Some(acronyms) = institution.display_name_acronyms {
            for acronym in acronyms {
                if !alternate_names.contains(&acronym) {
                    alternate_names.push(acronym);
                }
            }
        }
        let alternate_names = (!alternate_names.is_empty()).then_some(alternate_names);

        // Map image_url to organization options images
        let images = institution.image_url.map(|image_url| {
            vec![ImageObject {
                content_url: image_url,
                ..Default::default()
            }]
        });

        // Map ids to identifiers
        let identifiers = institution.ids.as_ref().and_then(|ids| {
            let mut id_map = IndexMap::new();
            if let Some(openalex) = &ids.openalex {
                id_map.insert("openalex".to_string(), openalex.clone());
            }
            if let Some(ror) = &ids.ror {
                id_map.insert("ror".to_string(), ror.clone());
            }
            if let Some(grid) = &ids.grid {
                id_map.insert("grid".to_string(), grid.clone());
            }
            if let Some(wikipedia) = &ids.wikipedia {
                id_map.insert("wikipedia".to_string(), wikipedia.clone());
            }
            if let Some(wikidata) = &ids.wikidata {
                id_map.insert("wikidata".to_string(), wikidata.clone());
            }
            if let Some(mag) = &ids.mag {
                id_map.insert("mag".to_string(), mag.clone());
            }
            convert_ids_to_identifiers(&id_map)
        });

        Organization {
            id: Some(institution.id),
            name: institution.display_name,
            ror,
            options: Box::new(OrganizationOptions {
                url: institution.homepage_url,
                alternate_names,
                images,
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Institution> for Node {
    fn from(inst: Institution) -> Self {
        Node::Organization(inst.into())
    }
}
