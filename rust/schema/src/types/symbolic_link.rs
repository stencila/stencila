// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A symbolic link on a file system.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("SymbolicLink")]
pub struct SymbolicLink {
    /// The type of this item.
    pub r#type: MustBe!("SymbolicLink"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The name of the symbolic link.
    pub name: String,

    /// The path (absolute or relative) of the symbolic link on the file system.
    pub path: String,

    /// The raw target path stored by the symbolic link.
    pub target: String,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl SymbolicLink {
    const NICK: [u8; 3] = *b"sym";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::SymbolicLink
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, path: String, target: String) -> Self {
        Self {
            name,
            path,
            target,
            ..Default::default()
        }
    }
}
