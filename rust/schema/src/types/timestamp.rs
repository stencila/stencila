// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::time_unit::TimeUnit;

/// A value that represents a point in time
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, HtmlCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Timestamp {
    /// The type of this item
    pub r#type: MustBe!("Timestamp"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z).
    pub value: Integer,

    /// The time unit that the `value` represents.
    pub time_unit: TimeUnit,
}
impl Timestamp {
    pub fn new(value: Integer, time_unit: TimeUnit) -> Self {
        Self {
            value,
            time_unit,
            ..Default::default()
        }
    }
}
