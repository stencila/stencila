//! Generated file, do not edit

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A schema specifying that a node must be one of several values.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct EnumValidator {
    /// The type of this item
    r#type: MustBe!("EnumValidator"),

    /// The identifier for this item
    id: String,

    /// A node is valid if it is equal to any of these values.
    values: Vec<Node>,
}
