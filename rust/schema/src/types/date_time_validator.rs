//! Generated file, do not edit

use crate::prelude::*;

use super::date_time::DateTime;
use super::string::String;

/// A validator specifying the constraints on a date-time.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct DateTimeValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("DateTimeValidator"),

    /// The identifier for this item
    id: Option<String>,

    /// The inclusive lower limit for a date-time.
    minimum: Option<DateTime>,

    /// The inclusive upper limit for a date-time.
    maximum: Option<DateTime>,
}
