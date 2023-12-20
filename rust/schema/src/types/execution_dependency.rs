// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::code_location::CodeLocation;
use super::execution_dependency_node::ExecutionDependencyNode;
use super::execution_dependency_relation::ExecutionDependencyRelation;
use super::string::String;

/// An upstream execution dependency of a node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ExecutionDependency")]
pub struct ExecutionDependency {
    /// The type of this item.
    pub r#type: MustBe!("ExecutionDependency"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The relation to the dependency.
    #[serde(alias = "dependency-relation", alias = "dependency_relation")]
    pub dependency_relation: ExecutionDependencyRelation,

    /// The node that is the dependency.
    #[serde(alias = "dependency-node", alias = "dependency_node")]
    pub dependency_node: ExecutionDependencyNode,

    /// The location that the dependency is defined.
    #[serde(alias = "code-location", alias = "code_location")]
    pub code_location: Option<CodeLocation>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl ExecutionDependency {
    pub fn new(dependency_relation: ExecutionDependencyRelation, dependency_node: ExecutionDependencyNode) -> Self {
        Self {
            dependency_relation,
            dependency_node,
            ..Default::default()
        }
    }
}

impl Entity for ExecutionDependency {
    const NICK: &'static str = "exe";

    fn node_type(&self) -> NodeType {
        NodeType::ExecutionDependency
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
