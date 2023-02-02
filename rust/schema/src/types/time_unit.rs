//! Generated file, do not edit

use crate::prelude::*;

/// A unit in which time can be measured
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
#[def = "Millisecond"]
pub enum TimeUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    Picosecond,
    Femtosecond,
    Attosecond,
}
