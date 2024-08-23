// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::cord::Cord;
use super::provenance_count::ProvenanceCount;
use super::string::String;

/// Document content in a specific format
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "RawBlock")]
#[patch(authors_on = "options")]
pub struct RawBlock {
    /// The type of this item.
    pub r#type: MustBe!("RawBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The format of the raw content.
    #[patch(format = "md", format = "myst")]
    pub format: String,

    /// The raw content.
    #[walk]
    #[patch(format = "md", format = "myst")]
    pub content: Cord,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<RawBlockOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct RawBlockOptions {
    /// The authors of the content.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[dom(elem = "span")]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[dom(elem = "span")]
    pub provenance: Option<Vec<ProvenanceCount>>,
}

impl RawBlock {
    const NICK: [u8; 3] = [114, 97, 119];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::RawBlock
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(format: String, content: Cord) -> Self {
        Self {
            format,
            content,
            ..Default::default()
        }
    }
}
