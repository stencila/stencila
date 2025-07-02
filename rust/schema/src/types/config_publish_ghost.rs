// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::config_publish_ghost_state::ConfigPublishGhostState;
use super::config_publish_ghost_type::ConfigPublishGhostType;
use super::date::Date;
use super::string::String;

/// Ghost publishing options.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("ConfigPublishGhost")]
pub struct ConfigPublishGhost {
    /// The type of Ghost resource (page or post).
    pub r#type: Option<ConfigPublishGhostType>,

    /// The URL slug for the page or post.
    #[patch(format = "all")]
    pub slug: Option<String>,

    /// Whether the page or post is featured.
    #[patch(format = "all")]
    pub featured: Option<Boolean>,

    /// The date that the page or post is to be published.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[patch(format = "all")]
    pub schedule: Option<Date>,

    /// the state of the page or post eg draft or published.
    #[patch(format = "all")]
    pub state: Option<ConfigPublishGhostState>,

    /// ghost tags.
    #[serde(alias = "tag")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[patch(format = "all")]
    pub tags: Option<Vec<String>>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ConfigPublishGhost {
    const NICK: [u8; 3] = *b"cpg";
    
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
