// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::config_publish_zenodo_access_right::ConfigPublishZenodoAccessRight;
use super::date::Date;
use super::string::String;

/// Zenodo publishing options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("ConfigPublishZenodo")]
pub struct ConfigPublishZenodo {
    /// The date of embargoed.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[patch(format = "all")]
    pub embargoed: Option<Date>,

    /// The access right of the document.
    #[serde(alias = "access-right")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[patch(format = "all")]
    pub access_right: Option<ConfigPublishZenodoAccessRight>,

    /// extra notes about deposition.
    #[patch(format = "all")]
    pub notes: Option<String>,

    /// The methodology of the study.
    #[patch(format = "all")]
    pub method: Option<String>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ConfigPublishZenodo {
    const NICK: [u8; 3] = *b"cpz";
    
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
