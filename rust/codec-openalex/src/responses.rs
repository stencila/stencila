use codec::common::serde::Deserialize;

use crate::{author::Author, institution::Institution, work::Work};

/// The response from getting a single entity
///
/// See https://docs.openalex.org/how-to-use-the-api/get-single-entities
#[derive(Deserialize)]
#[serde(crate = "codec::common::serde")]
pub struct SingleResponse<T> {
    #[serde(flatten)]
    #[allow(dead_code)]
    pub object: T,
}

/// The response from getting a list of entities
///
/// See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities
#[derive(Deserialize)]
#[serde(crate = "codec::common::serde")]
pub struct ListResponse<T> {
    pub results: Vec<T>,
    #[allow(dead_code)]
    pub meta: Option<Meta>,
}

#[derive(Deserialize)]
#[serde(crate = "codec::common::serde")]
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

/// Response for institutions API calls
pub type InstitutionsResponse = ListResponse<Institution>;
