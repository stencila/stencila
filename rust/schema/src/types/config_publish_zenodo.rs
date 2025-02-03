// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;

/// Zenodo publishing options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("ConfigPublishZenodo")]
pub struct ConfigPublishZenodo {
    /// Whether the deposit is embargoed.
    #[patch(format = "all")]
    pub embargoed: Option<Boolean>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ConfigPublishZenodo {
    const NICK: [u8; 3] = [99, 112, 122];
    
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
