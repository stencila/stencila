// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A schema specifying that a node must be a boolean value.
#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct BooleanValidator {
    /// The type of this item
    pub r#type: MustBe!("BooleanValidator"),

    /// The identifier for this item
    pub id: Option<String>,
}

impl BooleanValidator {
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
