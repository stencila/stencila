//! Generated file, do not edit

use crate::prelude::*;

use super::execution_dependant_node::ExecutionDependantNode;
use super::execution_dependant_relation::ExecutionDependantRelation;
use super::integer::Integer;

/// A downstream execution dependant of a node
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionDependant {
    /// The relation to the dependant
    pub dependant_relation: ExecutionDependantRelation,

    /// The node that is the dependant
    pub dependant_node: ExecutionDependantNode,

    /// The location that the dependant is defined within code
    pub code_location: Option<Vec<Integer>>,
}
