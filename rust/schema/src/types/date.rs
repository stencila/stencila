//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A calendar date encoded as a ISO 8601 string.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Date {
    /// The date as an ISO 8601 string.
    value: String,
}
