// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A calendar date encoded as a ISO 8601 string.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Date {
    /// The type of this item
    pub r#type: MustBe!("Date"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    pub value: String,
}

impl Date {
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
