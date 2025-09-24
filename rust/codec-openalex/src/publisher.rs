use serde::Deserialize;

use stencila_codec::stencila_schema::{ImageObject, Node, Organization, OrganizationOptions};

use crate::{
    ids::{Ids, ids_get_maybe, ids_to_identifiers},
    utils::strip_ror_prefix,
};

/// An OpenAlex `Publisher` object
///
/// See https://docs.openalex.org/api-entities/publishers/publisher-object
///
/// Fields not currently used are commented out to reduce bloat and avoid risk
/// or deserialization errors.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Publisher {
    pub id: String,
    pub display_name: Option<String>,
    pub alternate_titles: Option<Vec<String>>,
    //pub hierarchy_level: Option<i32>,
    //pub parent_publisher: Option<String>,
    //pub lineage: Option<Vec<String>>,
    //pub country_codes: Option<Vec<String>>,
    pub homepage_url: Option<String>,
    pub image_url: Option<String>,
    //pub image_thumbnail_url: Option<String>,
    //pub works_count: Option<i64>,
    //pub cited_by_count: Option<i64>,
    //pub sources_api_url: Option<String>,
    //pub summary_stats: Option<SummaryStats>,
    pub ids: Option<Ids>,
    //pub counts_by_year: Option<Vec<CountsByYear>>,
    //pub updated_date: Option<String>,
    //pub created_date: Option<String>,
    //pub roles: Option<Vec<Role>>,
}

impl From<Publisher> for Organization {
    fn from(publisher: Publisher) -> Self {
        let ror = strip_ror_prefix(
            publisher
                .ids
                .as_ref()
                .and_then(|ids| ids_get_maybe(ids, "ror")),
        );

        let name = publisher.display_name;

        let alternate_names = publisher.alternate_titles.filter(|names| !names.is_empty());

        let url = publisher.homepage_url;

        let images = publisher.image_url.map(|image_url| {
            vec![ImageObject {
                content_url: image_url,
                ..Default::default()
            }]
        });

        let identifiers = publisher.ids.and_then(ids_to_identifiers);

        Organization {
            ror,
            name,
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

impl From<Publisher> for Node {
    fn from(publisher: Publisher) -> Self {
        Node::Organization(publisher.into())
    }
}
