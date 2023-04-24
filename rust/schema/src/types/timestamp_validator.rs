// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::time_unit::TimeUnit;
use super::timestamp::Timestamp;

/// A validator specifying the constraints on a timestamp.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TimestampValidator {
    /// The type of this item
    pub r#type: MustBe!("TimestampValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The time units that the timestamp can have.
    pub time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a timestamp.
    pub minimum: Option<Timestamp>,

    /// The inclusive upper limit for a timestamp.
    pub maximum: Option<Timestamp>,
}

impl TimestampValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
