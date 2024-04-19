// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A file on the file system.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "File")]
pub struct File {
    /// The type of this item.
    pub r#type: MustBe!("File"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the file.
    pub name: String,

    /// The path (absolute or relative) of the file on the file system
    pub path: String,

    /// IANA media type (MIME type).
    #[serde(alias = "encodingFormat", alias = "media-type", alias = "media_type")]
    pub media_type: Option<String>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl File {
    const NICK: [u8; 3] = [102, 105, 108];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::File
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            ..Default::default()
        }
    }
}
