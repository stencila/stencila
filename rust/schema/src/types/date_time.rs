// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DateTime {
    /// The type of this item
    pub r#type: MustBe!("DateTime"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    pub value: String,
}

impl DateTime {
    #[rustfmt::skip]
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
