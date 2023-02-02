//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::time_unit::TimeUnit;
use super::timestamp::Timestamp;

/// A validator specifying the constraints on a timestamp.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct TimestampValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("TimestampValidator"),

    /// The identifier for this item
    id: String,

    /// The time units that the timestamp can have.
    time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a timestamp.
    minimum: Option<Timestamp>,

    /// The inclusive upper limit for a timestamp.
    maximum: Option<Timestamp>,
}
