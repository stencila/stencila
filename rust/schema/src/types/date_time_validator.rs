//! Generated file, do not edit

use crate::prelude::*;

use super::date_time::DateTime;
use super::string::String;

/// A validator specifying the constraints on a date-time.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DateTimeValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("DateTimeValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The inclusive lower limit for a date-time.
    pub minimum: Option<DateTime>,

    /// The inclusive upper limit for a date-time.
    pub maximum: Option<DateTime>,
}
