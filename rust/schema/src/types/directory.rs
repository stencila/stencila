// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::file_or_directory::FileOrDirectory;
use super::string::String;

/// A directory on the file system.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Directory")]
pub struct Directory {
    /// The type of this item.
    pub r#type: MustBe!("Directory"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the directory.
    pub name: String,

    /// The path (absolute or relative) of the file on the file system.
    pub path: String,

    /// The files and other directories within this directory.
    #[serde(alias = "hasParts", alias = "part")]
    #[serde(deserialize_with = "one_or_many")]
    #[dom(elem = "div")]
    pub parts: Vec<FileOrDirectory>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Directory {
    const NICK: [u8; 3] = [100, 105, 114];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Directory
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, path: String, parts: Vec<FileOrDirectory>) -> Self {
        Self {
            name,
            path,
            parts,
            ..Default::default()
        }
    }
}
