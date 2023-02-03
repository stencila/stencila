//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::time_unit::TimeUnit;
use super::timestamp::Timestamp;

/// A validator specifying the constraints on a timestamp.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TimestampValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("TimestampValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The time units that the timestamp can have.
    pub time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a timestamp.
    pub minimum: Option<Timestamp>,

    /// The inclusive upper limit for a timestamp.
    pub maximum: Option<Timestamp>,
}
