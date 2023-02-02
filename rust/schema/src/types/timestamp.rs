//! Generated file, do not edit

use crate::prelude::*;

use super::integer::Integer;
use super::time_unit::TimeUnit;

/// A value that represents a point in time
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Timestamp {
    /// The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z).
    value: Integer,

    /// The time unit that the `value` represents.
    time_unit: TimeUnit,
}
