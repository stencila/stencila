// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::execution_bounds::ExecutionBounds;
use super::number::Number;

/// Model selection and execution options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("ConfigModels")]
pub struct ConfigModels {
    /// Automatically execute generated content.
    #[serde(alias = "execute-content", alias = "execute_content")]
    #[patch(format = "all")]
    pub execute_content: Option<Boolean>,

    /// The execution boundaries on model generated code.
    #[serde(alias = "execution-bounds", alias = "execution_bounds")]
    #[patch(format = "all")]
    pub execution_bounds: Option<ExecutionBounds>,

    /// When executing model generated content, the maximum number of retries.
    #[serde(alias = "max-retries", alias = "maximum-retries", alias = "execution-retries", alias = "retries", alias = "maximum_retries")]
    #[patch(format = "all")]
    pub maximum_retries: Option<Number>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ConfigModels {
    const NICK: [u8; 3] = *b"cfm";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Config
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
