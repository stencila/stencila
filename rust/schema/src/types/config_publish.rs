// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::config_publish_ghost::ConfigPublishGhost;
use super::config_publish_zenodo::ConfigPublishZenodo;

/// Publishing options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("ConfigPublish")]
pub struct ConfigPublish {
    /// Ghost publishing options.
    #[patch(format = "all")]
    pub ghost: Option<ConfigPublishGhost>,

    /// Zenodo publishing options.
    #[patch(format = "all")]
    pub zenodo: Option<ConfigPublishZenodo>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ConfigPublish {
    const NICK: [u8; 3] = *b"cfp";
    
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
