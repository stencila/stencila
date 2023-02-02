//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::time::Time;

/// A validator specifying the constraints on a time.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct TimeValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("TimeValidator"),

    /// The identifier for this item
    id: Option<String>,

    /// The inclusive lower limit for a time.
    minimum: Option<Time>,

    /// The inclusive upper limit for a time.
    maximum: Option<Time>,
}
