//! Generated file, do not edit

use crate::prelude::*;

use super::execution_dependency_node::ExecutionDependencyNode;
use super::execution_dependency_relation::ExecutionDependencyRelation;
use super::integer::Integer;

/// An upstream execution dependency of a node
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionDependency {
    /// The relation to the dependency
    pub dependency_relation: ExecutionDependencyRelation,

    /// The node that is the dependency
    pub dependency_node: ExecutionDependencyNode,

    /// The location that the dependency is defined within code
    pub code_location: Option<Vec<Integer>>,
}

impl ExecutionDependency {
    pub fn new(dependency_relation: ExecutionDependencyRelation, dependency_node: ExecutionDependencyNode) -> Self {
        Self{
            dependency_relation,
            dependency_node,
            ..Default::default()
        }
    }
}

