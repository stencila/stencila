use codec::{common::serde::Deserialize, schema::{ImageObject, Organization}};

/// An OpenAlex `Publisher` object
///
/// See https://docs.openalex.org/api-entities/publishers/publisher-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Publisher {
    pub id: String,
    pub display_name: Option<String>,
    pub alternate_titles: Option<Vec<String>>,
    pub hierarchy_level: Option<i32>,
    pub parent_publisher: Option<String>,
    pub lineage: Option<Vec<String>>,
    pub country_codes: Option<Vec<String>>,
    pub homepage_url: Option<String>,
    pub image_url: Option<String>,
    pub image_thumbnail_url: Option<String>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub sources_api_url: Option<String>,
    pub summary_stats: Option<SummaryStats>,
    pub ids: Option<ExternalIds>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,
    pub roles: Option<Vec<String>>,
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
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
}

impl From<Publisher> for Organization {
    fn from(publisher: Publisher) -> Self {
        let mut organization = Organization {
            id: Some(publisher.id),
            name: publisher.display_name,
            ror: crate::strip_ror_prefix(publisher.ids.and_then(|ids| ids.ror)),
            ..Default::default()
        };

        // Map homepage_url to organization options url
        organization.options.url = publisher.homepage_url;

        // Map image_url to organization options images
        if let Some(image_url) = publisher.image_url {
            organization.options.images = Some(vec![ImageObject {
                content_url: image_url,
                ..Default::default()
            }]);
        }

        organization
    }
}
