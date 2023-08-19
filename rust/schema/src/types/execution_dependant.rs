// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::execution_dependant_node::ExecutionDependantNode;
use super::execution_dependant_relation::ExecutionDependantRelation;
use super::integer::Integer;

/// A downstream execution dependant of a node
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionDependant {
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
