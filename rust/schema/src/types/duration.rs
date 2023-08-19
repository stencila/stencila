// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::time_unit::TimeUnit;

/// A value that represents the difference between two timestamps
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Duration {
    /// The type of this item
    pub r#type: MustBe!("Duration"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The time difference in `timeUnit`s.
    pub value: Integer,

    /// The time unit that the `value` represents.
    pub time_unit: TimeUnit,
}
impl Duration {
    pub fn new(value: Integer, time_unit: TimeUnit) -> Self {
        Self {
            value,
            time_unit,
            ..Default::default()
        }
    }
}
