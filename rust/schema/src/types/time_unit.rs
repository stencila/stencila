// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// A unit in which time can be measured
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml, ToText)]
#[serde(crate = "common::serde")]
pub enum TimeUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    #[default]
    Millisecond,
    Microsecond,
    Nanosecond,
    Picosecond,
    Femtosecond,
    Attosecond,
}
