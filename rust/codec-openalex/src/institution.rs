use serde::Deserialize;

use stencila_codec::stencila_schema::{ImageObject, Node, Organization, OrganizationOptions};

use crate::{
    ids::{Ids, id_to_identifiers, ids_get_maybe, ids_to_identifiers},
    utils::{get_or_generate_ror, strip_ror_prefix},
};

/// An OpenAlex `Institution` object
///
/// See https://docs.openalex.org/api-entities/institutions/institution-object
///
/// Fields not currently used are commented out to reduce bloat and avoid risk
/// or deserialization errors.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Institution {
    pub id: String,
    pub ror: Option<String>,
    pub display_name: Option<String>,
    //pub country_code: Option<String>,
    //pub r#type: Option<String>,
    pub homepage_url: Option<String>,
    pub image_url: Option<String>,
    //pub image_thumbnail_url: Option<String>,
    pub display_name_acronyms: Option<Vec<String>>,
    pub display_name_alternatives: Option<Vec<String>>,
    //pub works_count: Option<i64>,
    //pub cited_by_count: Option<i64>,
    //pub summary_stats: Option<SummaryStats>,
    pub ids: Option<Ids>,
    //pub geo: Option<Geo>,
    //pub international: Option<International>,
    //pub associated_institutions: Option<Vec<AssociatedInstitution>>,
    //pub counts_by_year: Option<Vec<CountsByYear>>,
    //pub works_api_url: Option<String>,
    //pub updated_date: Option<String>,
    //pub created_date: Option<String>,
    //pub lineage: Option<Vec<String>>,
    //pub roles: Option<Vec<Role>>,
    //pub x_concepts: Option<Vec<Concept>>,
}

impl Institution {
    /// Get the ROR of an institution, or generate a pseudo ROR
    pub fn ror(&self, prefix: char) -> String {
        get_or_generate_ror(&self.ror, &self.id, prefix)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedInstitution {
    pub id: Option<String>,
    pub ror: Option<String>,
    pub display_name: Option<String>,
    //pub country_code: Option<String>,
    //pub r#type: Option<String>,
    //pub lineage: Option<Vec<String>>,
}

impl DehydratedInstitution {
    /// Get the ROR of an institution, or generate a pseudo ROR
    pub fn ror(&self, prefix: char) -> String {
        if let Some(id) = &self.id {
            get_or_generate_ror(&self.ror, id, prefix)
        } else {
            format!("{prefix}unknown")
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct International {
    pub display_name: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct AssociatedInstitution {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub ror: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub relationship: Option<String>,
}

impl From<Institution> for Organization {
    fn from(institution: Institution) -> Self {
        let ror = strip_ror_prefix(
            institution.ror.clone().or(institution
                .ids
                .as_ref()
                .and_then(|ids| ids_get_maybe(ids, "ror"))),
        );

        let name = institution.display_name;

        let mut alternate_names = institution.display_name_alternatives.unwrap_or_default();
        if let Some(acronyms) = institution.display_name_acronyms {
            for acronym in acronyms {
                if !alternate_names.contains(&acronym) {
                    alternate_names.push(acronym);
                }
            }
        }
        let alternate_names = (!alternate_names.is_empty()).then_some(alternate_names);

        let url = institution.homepage_url;

        let images = institution
            .image_url
            .map(|image_url| vec![ImageObject::new(image_url)]);

        let identifiers = institution.ids.and_then(ids_to_identifiers);

        Organization {
            name,
            ror,
            options: Box::new(OrganizationOptions {
                alternate_names,
                url,
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

impl From<DehydratedInstitution> for Organization {
    fn from(institution: DehydratedInstitution) -> Self {
        let ror = strip_ror_prefix(institution.ror);

        let name = institution.display_name;

        let identifiers = institution
            .id
            .and_then(|id| id_to_identifiers("openalex", id));

        Self {
            ror,
            name,
            options: Box::new(OrganizationOptions {
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
