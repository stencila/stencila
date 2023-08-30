// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A schema specifying that a node must be one of several values.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct EnumValidator {
    /// The type of this item
    pub r#type: MustBe!("EnumValidator"),

    /// The identifier for this item
    #[strip(id)]
    pub id: Option<String>,

    /// A node is valid if it is equal to any of these values.
    pub values: Vec<Node>,
}
impl EnumValidator {
    pub fn new(values: Vec<Node>) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }
}
