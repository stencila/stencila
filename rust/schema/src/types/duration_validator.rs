// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::duration::Duration;
use super::string::String;
use super::time_unit::TimeUnit;

/// A validator specifying the constraints on a duration.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DurationValidator {
    /// The type of this item
    pub r#type: MustBe!("DurationValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The time units that the duration can have.
    pub time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a duration.
    pub minimum: Option<Duration>,

    /// The inclusive upper limit for a duration.
    pub maximum: Option<Duration>,
}

impl DurationValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
