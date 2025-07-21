use codec::{
    common::{serde::Deserialize, serde_json},
    schema::{ImageObject, Organization},
};

/// An OpenAlex `Institution` object
///
/// See https://docs.openalex.org/api-entities/institutions/institution-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
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
    pub grid: Option<String>,
    pub wikipedia: Option<String>,
    pub wikidata: Option<String>,
    pub mag: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
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
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct International {
    pub display_name: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct AssociatedInstitution {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub ror: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub relationship: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
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
pub struct Concept {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}

impl From<Institution> for Organization {
    fn from(institution: Institution) -> Self {
        let mut organization = Organization {
            id: Some(institution.id),
            ror: crate::strip_ror_prefix(institution.ror),
            name: institution.display_name,
            ..Default::default()
        };

        // Map homepage_url to organization options url
        organization.options.url = institution.homepage_url;

        // Map image_url to organization options images
        if let Some(image_url) = institution.image_url {
            organization.options.images = Some(vec![ImageObject {
                content_url: image_url,
                ..Default::default()
            }]);
        }

        organization
    }
}
