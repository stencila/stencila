//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A schema specifying that a node must be a boolean value.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct BooleanValidator {
    /// The type of this item
    r#type: MustBe!("BooleanValidator"),

    /// The identifier for this item
    id: String,
}
