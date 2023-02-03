//! Generated file, do not edit

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// A tag on code that affects its execution
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionTag {
    /// The name of the tag
    pub name: String,

    /// The value of the tag
    pub value: String,

    /// Whether the tag is global to the document
    pub is_global: Boolean,
}
