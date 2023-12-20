// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// A digest of the content, semantics and dependencies of an executable node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CompilationDigest")]
pub struct CompilationDigest {
    /// The type of this item.
    pub r#type: MustBe!("CompilationDigest"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A digest of the state of a node.
    #[serde(alias = "state-digest", alias = "state_digest")]
    pub state_digest: UnsignedInteger,

    /// A digest of the semantics of the node with respect to the dependency graph.
    #[serde(alias = "semantic-digest", alias = "semantic_digest")]
    pub semantic_digest: Option<UnsignedInteger>,

    /// A digest of the semantic digests of the dependencies of a node.
    #[serde(alias = "dependencies-digest", alias = "dependencies_digest")]
    pub dependencies_digest: Option<UnsignedInteger>,

    /// A count of the number of dependencies that are stale.
    #[serde(alias = "dependencies-stale", alias = "dependencies_stale")]
    pub dependencies_stale: Option<UnsignedInteger>,

    /// A count of the number of dependencies that failed.
    #[serde(alias = "dependencies-failed", alias = "dependencies_failed")]
    pub dependencies_failed: Option<UnsignedInteger>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl CompilationDigest {
    pub fn new(state_digest: UnsignedInteger) -> Self {
        Self {
            state_digest,
            ..Default::default()
        }
    }
}

impl Entity for CompilationDigest {
    const NICK: &'static str = "com";

    fn node_type(&self) -> NodeType {
        NodeType::CompilationDigest
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
