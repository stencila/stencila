use serde::Deserialize;

use stencila_codec::stencila_schema::{ImageObject, Node, Organization, OrganizationOptions};

use crate::{
    ids::{Ids, ids_get_maybe, ids_to_identifiers},
    utils::strip_ror_prefix,
};

/// An OpenAlex `Funder` object
///
/// See https://docs.openalex.org/api-entities/funders/funder-object
///
/// Fields not currently used are commented out to reduce bloat and avoid risk
/// or deserialization errors.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Funder {
    pub id: String,
    pub display_name: Option<String>,
    pub alternate_titles: Option<Vec<String>>,
    //pub country_code: Option<String>,
    //pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub image_url: Option<String>,
    //pub image_thumbnail_url: Option<String>,
    //pub grants_count: Option<i64>,
    //pub works_count: Option<i64>,
    //pub cited_by_count: Option<i64>,
    //pub summary_stats: Option<SummaryStats>,
    pub ids: Option<Ids>,
    //pub counts_by_year: Option<Vec<CountsByYear>>,
    //pub updated_date: Option<String>,
    //pub created_date: Option<String>,
    //pub roles: Option<Vec<Role>>,
}

impl From<Funder> for Organization {
    fn from(funder: Funder) -> Self {
        let ror = strip_ror_prefix(
            funder
                .ids
                .as_ref()
                .and_then(|ids| ids_get_maybe(ids, "ror")),
        );

        let name = funder.display_name;

        let alternate_names = funder.alternate_titles.filter(|names| !names.is_empty());

        let url = funder.homepage_url;

        let images = funder
            .image_url
            .map(|image_url| vec![ImageObject::new(image_url)]);

        let identifiers = funder.ids.and_then(ids_to_identifiers);

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

impl From<Funder> for Node {
    fn from(funder: Funder) -> Self {
        Node::Organization(funder.into())
    }
}
