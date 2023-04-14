// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::null::Null;
use super::string::String;

/// A validator specifying a constant value that a node must have.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ConstantValidator {
    /// The type of this item
    pub r#type: MustBe!("ConstantValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The value that the node must have.
    #[def = "Box::new(Node::Null(Null{}))"]
    pub value: Box<Node>,
}

impl ConstantValidator {
    #[rustfmt::skip]
    pub fn new(value: Box<Node>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
