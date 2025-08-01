use serde::Deserialize;

use codec::common::serde_json;

use crate::{
    author::Author, funder::Funder, institution::Institution, publisher::Publisher, source::Source,
    work::Work,
};

/// The response from getting a single entity
///
/// See https://docs.openalex.org/how-to-use-the-api/get-single-entities
#[derive(Deserialize)]
pub struct SingleResponse<T> {
    #[serde(flatten)]
    #[allow(dead_code)]
    pub object: T,
}

/// The response from getting a list of entities
///
/// See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities
#[derive(Deserialize)]
pub struct ListResponse<T> {
    pub results: Vec<T>,
    #[allow(dead_code)]
    pub meta: Option<Meta>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Meta {
    pub count: Option<i64>,
    pub db_response_time_ms: Option<i32>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub groups_count: Option<i32>,
}

/// Response for works API calls
pub type WorksResponse = ListResponse<Work>;

/// Response for authors API calls  
pub type AuthorsResponse = ListResponse<Author>;

/// Response for sources API calls
pub type SourcesResponse = ListResponse<Source>;

/// Response for institutions API calls
pub type InstitutionsResponse = ListResponse<Institution>;

/// Response for publishers API calls
pub type PublishersResponse = ListResponse<Publisher>;

/// Response for funder API calls
pub type FundersResponse = ListResponse<Funder>;

/// Response for select API calls with partial fields
///
/// See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities/select-fields
pub type SelectResponse = ListResponse<serde_json::Map<String, serde_json::Value>>;

/// Response for a call with select=id
#[derive(Deserialize)]
#[serde(crate = "codec::common::serde")]
pub struct Id {
    pub id: String,
}
pub type IdResponse = ListResponse<Id>;
