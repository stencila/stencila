// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// A tag on code that affects its execution
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionTag {
    /// The name of the tag
    pub name: String,

    /// The value of the tag
    pub value: String,

    /// Whether the tag is global to the document
    pub is_global: Boolean,
}

impl ExecutionTag {
    #[rustfmt::skip]
    pub fn new(name: String, value: String, is_global: Boolean) -> Self {
        Self {
            name,
            value,
            is_global
        }
    }
}
