//! Generated file, do not edit

use crate::prelude::*;

use super::node::Node;
use super::null::Null;
use super::string::String;

/// A validator specifying a constant value that a node must have.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ConstantValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("ConstantValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The value that the node must have.
    #[def = "Box::new(Node::Null(Null{}))"]
    pub value: Box<Node>,
}
