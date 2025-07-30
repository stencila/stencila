use crate::{strip_ror_prefix, utils::convert_ids_to_identifiers};
use codec::{
    common::{indexmap::IndexMap, serde::Deserialize},
    schema::{ImageObject, Node, Organization, OrganizationOptions},
};

/// An OpenAlex `Funder` object
///
/// See https://docs.openalex.org/api-entities/funders/funder-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Funder {
    pub id: String,
    pub display_name: Option<String>,
    pub alternate_titles: Option<Vec<String>>,
    pub country_code: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub image_url: Option<String>,
    pub image_thumbnail_url: Option<String>,
    pub grants_count: Option<i64>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub summary_stats: Option<SummaryStats>,
    pub ids: Option<ExternalIds>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,
    pub roles: Option<Vec<Role>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Role {
    pub role: Option<String>,
    pub id: Option<String>,
    pub works_count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct SummaryStats {
    #[serde(rename = "2yr_mean_citedness")]
    pub two_yr_mean_citedness: Option<f64>,
    pub h_index: Option<i32>,
    pub i10_index: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct ExternalIds {
    pub openalex: Option<String>,
    pub ror: Option<String>,
    pub wikidata: Option<String>,
    pub crossref: Option<String>,
    pub doi: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
}

impl From<Funder> for Organization {
    fn from(funder: Funder) -> Self {
        // Get ROR
        let ror = strip_ror_prefix(funder.ids.as_ref().and_then(|ids| ids.ror.clone()));

        // Map image_url to organization options images
        let images = funder.image_url.map(|image_url| {
            vec![ImageObject {
                content_url: image_url,
                ..Default::default()
            }]
        });

        // Map ids to identifiers
        let identifiers = funder.ids.as_ref().and_then(|ids| {
            let mut id_map = IndexMap::new();
            if let Some(openalex) = &ids.openalex {
                id_map.insert("openalex".to_string(), openalex.clone());
            }
            if let Some(ror) = &ids.ror {
                id_map.insert("ror".to_string(), ror.clone());
            }
            if let Some(wikidata) = &ids.wikidata {
                id_map.insert("wikidata".to_string(), wikidata.clone());
            }
            if let Some(crossref) = &ids.crossref {
                id_map.insert("crossref".to_string(), crossref.clone());
            }
            if let Some(doi) = &ids.doi {
                id_map.insert("doi".to_string(), doi.clone());
            }
            convert_ids_to_identifiers(&id_map)
        });

        Organization {
            id: Some(funder.id),
            name: funder.display_name,
            ror,
            options: Box::new(OrganizationOptions {
                url: funder.homepage_url,
                images,
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Funder> for Node {
    fn from(funder: Funder) -> Self {
        Node::Organization(funder.into())
    }
}
