//! Generated file, do not edit

use crate::prelude::*;

use super::integer::Integer;
use super::time_unit::TimeUnit;

/// A value that represents the difference between two timestamps
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Duration {
    /// The time difference in `timeUnit`s.
    value: Integer,

    /// The time unit that the `value` represents.
    time_unit: TimeUnit,
}
