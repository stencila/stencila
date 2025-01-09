// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::inline::Inline;
use super::string::String;

/// Annotated content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Annotation")]
#[jats(elem = "annotation")]
#[markdown(template = "=={{content}}==", escape = "=")]
pub struct Annotation {
    /// The type of this item.
    pub r#type: MustBe!("Annotation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is marked.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "none")]
    pub content: Vec<Inline>,

    /// The annotation, usually a paragraph.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[dom(elem = "aside")]
    pub annotation: Option<Vec<Block>>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Annotation {
    const NICK: [u8; 3] = [97, 110, 110];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Annotation
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
