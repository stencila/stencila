//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::time::Time;

/// A validator specifying the constraints on a time.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TimeValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("TimeValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The inclusive lower limit for a time.
    pub minimum: Option<Time>,

    /// The inclusive upper limit for a time.
    pub maximum: Option<Time>,
}
