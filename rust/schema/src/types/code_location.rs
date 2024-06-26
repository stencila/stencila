// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// The location within some source code.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CodeLocation")]
pub struct CodeLocation {
    /// The type of this item.
    pub r#type: MustBe!("CodeLocation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The source of the code, a file path, label or URL.
    pub source: Option<String>,

    /// The 0-based index if the first line on which the error occurred.
    #[serde(alias = "start-line", alias = "start_line")]
    pub start_line: Option<UnsignedInteger>,

    /// The 0-based index if the first column on which the error occurred.
    #[serde(alias = "start-column", alias = "start_column")]
    pub start_column: Option<UnsignedInteger>,

    /// The 0-based index if the last line on which the error occurred.
    #[serde(alias = "end-line", alias = "end_line")]
    pub end_line: Option<UnsignedInteger>,

    /// The 0-based index if the last column on which the error occurred.
    #[serde(alias = "end-column", alias = "end_column")]
    pub end_column: Option<UnsignedInteger>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl CodeLocation {
    const NICK: [u8; 3] = [99, 100, 108];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::CodeLocation
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
