// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A schema specifying that a node must be one of several values.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct EnumValidator {
    /// The type of this item
    pub r#type: MustBe!("EnumValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// A node is valid if it is equal to any of these values.
    pub values: Vec<Node>,
}

impl EnumValidator {
    #[rustfmt::skip]
    pub fn new(values: Vec<Node>) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }
}
