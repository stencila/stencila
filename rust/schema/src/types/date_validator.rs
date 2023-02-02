//! Generated file, do not edit

use crate::prelude::*;

use super::date::Date;
use super::string::String;

/// A validator specifying the constraints on a date.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct DateValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("DateValidator"),

    /// The identifier for this item
    id: Option<String>,

    /// The inclusive lower limit for a date.
    minimum: Option<Date>,

    /// The inclusive upper limit for a date.
    maximum: Option<Date>,
}
