//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A calendar date encoded as a ISO 8601 string.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Date {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Date"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    pub value: String,
}
