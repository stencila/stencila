// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::config_publish::ConfigPublish;
use super::string::String;

/// Stencila document configuration options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("Config")]
pub struct Config {
    /// The styling theme to use for the document
    #[patch(format = "all")]
    pub theme: Option<String>,

    /// Publishing configuration options
    #[patch(format = "all")]
    pub publish: Option<ConfigPublish>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Config {
    const NICK: [u8; 3] = [99, 102, 103];
    
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
