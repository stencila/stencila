// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A validator specifying a constant value that a node must have.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, HtmlCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ConstantValidator {
    /// The type of this item
    pub r#type: MustBe!("ConstantValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The value that the node must have.
    pub value: Box<Node>,
}
impl ConstantValidator {
    pub fn new(value: Box<Node>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
