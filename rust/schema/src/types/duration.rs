//! Generated file, do not edit

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::time_unit::TimeUnit;

/// A value that represents the difference between two timestamps
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Duration {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Duration"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The time difference in `timeUnit`s.
    pub value: Integer,

    /// The time unit that the `value` represents.
    pub time_unit: TimeUnit,
}
