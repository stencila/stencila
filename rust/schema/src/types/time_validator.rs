//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::time::Time;

/// A validator specifying the constraints on a time.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TimeValidator {
    /// The type of this item
    r#type: MustBe!("TimeValidator"),

    /// The identifier for this item
    id: String,

    /// The inclusive lower limit for a time.
    minimum: Option<Time>,

    /// The inclusive upper limit for a time.
    maximum: Option<Time>,
}
