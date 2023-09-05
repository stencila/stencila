// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::execution_dependant_node::ExecutionDependantNode;
use super::execution_dependant_relation::ExecutionDependantRelation;
use super::integer::Integer;
use super::string::String;

/// A downstream execution dependant of a node
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionDependant {
    /// The type of this item
    pub r#type: MustBe!("ExecutionDependant"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The relation to the dependant
    pub dependant_relation: ExecutionDependantRelation,

    /// The node that is the dependant
    pub dependant_node: ExecutionDependantNode,

    /// The location that the dependant is defined within code
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
