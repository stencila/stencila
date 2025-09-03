// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::code_location::CodeLocation;
use super::execution_dependency_relation::ExecutionDependencyRelation;
use super::string::String;

/// An upstream execution dependency of a node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("ExecutionDependency")]
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

    /// The type of node that is the dependency.
    #[serde(alias = "dependency-type", alias = "dependency_type")]
    pub dependency_type: String,

    /// The id of node that is the dependency.
    #[serde(alias = "dependency-id", alias = "dependency_id")]
    pub dependency_id: String,

    /// The location that the dependency is defined.
    #[serde(alias = "code-location", alias = "code_location")]
    pub code_location: Option<CodeLocation>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ExecutionDependency {
    const NICK: [u8; 3] = *b"exy";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ExecutionDependency
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(dependency_relation: ExecutionDependencyRelation, dependency_type: String, dependency_id: String) -> Self {
        Self {
            dependency_relation,
            dependency_type,
            dependency_id,
            ..Default::default()
        }
    }
}
