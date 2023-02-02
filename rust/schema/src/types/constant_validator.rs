//! Generated file, do not edit

use crate::prelude::*;

use super::node::Node;
use super::null::Null;
use super::string::String;

/// A validator specifying a constant value that a node must have.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ConstantValidator {
    /// The type of this item
    r#type: MustBe!("ConstantValidator"),

    /// The identifier for this item
    id: String,

    /// The value that the node must have.
    #[def = "Box::new(Node::Null(Null))"]
    value: Box<Node>,
}
