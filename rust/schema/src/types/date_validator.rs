//! Generated file, do not edit

use crate::prelude::*;

use super::date::Date;
use super::string::String;

/// A validator specifying the constraints on a date.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DateValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("DateValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The inclusive lower limit for a date.
    pub minimum: Option<Date>,

    /// The inclusive upper limit for a date.
    pub maximum: Option<Date>,
}
