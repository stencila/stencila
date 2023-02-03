//! Generated file, do not edit

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::time_unit::TimeUnit;

/// A value that represents a point in time
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Timestamp {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Timestamp"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z).
    pub value: Integer,

    /// The time unit that the `value` represents.
    pub time_unit: TimeUnit,
}
