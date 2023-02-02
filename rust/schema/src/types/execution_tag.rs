//! Generated file, do not edit

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// A tag on code that affects its execution
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ExecutionTag {
    /// The name of the tag
    name: String,

    /// The value of the tag
    value: String,

    /// Whether the tag is global to the document
    is_global: Boolean,
}
