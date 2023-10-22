// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::execution_dependant_node::ExecutionDependantNode;
use super::execution_dependant_relation::ExecutionDependantRelation;
use super::integer::Integer;
use super::string::String;

/// A downstream execution dependant of a node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ExecutionDependant")]
pub struct ExecutionDependant {
    /// The type of this item.
    pub r#type: MustBe!("ExecutionDependant"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The relation to the dependant
    #[serde(alias = "dependant-relation", alias = "dependant_relation")]
    pub dependant_relation: ExecutionDependantRelation,

    /// The node that is the dependant
    #[serde(alias = "dependant-node", alias = "dependant_node")]
    pub dependant_node: ExecutionDependantNode,

    /// The location that the dependant is defined within code
    #[serde(alias = "code-location", alias = "code_location")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub code_location: Option<Vec<Integer>>,
}

impl ExecutionDependant {
    pub fn new(dependant_relation: ExecutionDependantRelation, dependant_node: ExecutionDependantNode) -> Self {
        Self {
            dependant_relation,
            dependant_node,
            ..Default::default()
        }
    }
}
