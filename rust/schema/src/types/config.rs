// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::config_models::ConfigModels;
use super::config_publish::ConfigPublish;
use super::string::String;

/// Stencila document configuration options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Config")]
pub struct Config {
    /// The styling theme for the document
    #[patch(format = "all")]
    pub theme: Option<String>,

    /// The parameters used for selecting and running generative AI models
    #[patch(format = "all")]
    pub models: Option<ConfigModels>,

    /// Publishing configuration options
    #[patch(format = "all")]
    pub publish: Option<ConfigPublish>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Config {
    const NICK: [u8; 3] = *b"cfg";
    
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
