//! Generated file, do not edit

use crate::prelude::*;

use super::duration::Duration;
use super::string::String;
use super::time_unit::TimeUnit;

/// A validator specifying the constraints on a duration.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DurationValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("DurationValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The time units that the duration can have.
    pub time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a duration.
    pub minimum: Option<Duration>,

    /// The inclusive upper limit for a duration.
    pub maximum: Option<Duration>,
}
