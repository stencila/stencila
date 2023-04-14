// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::date_time::DateTime;
use super::string::String;

/// A validator specifying the constraints on a date-time.
#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DateTimeValidator {
    /// The type of this item
    pub r#type: MustBe!("DateTimeValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The inclusive lower limit for a date-time.
    pub minimum: Option<DateTime>,

    /// The inclusive upper limit for a date-time.
    pub maximum: Option<DateTime>,
}

impl DateTimeValidator {
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
