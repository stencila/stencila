// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::time::Time;

/// A validator specifying the constraints on a time.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("TimeValidator")]
pub struct TimeValidator {
    /// The type of this item.
    pub r#type: MustBe!("TimeValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The inclusive lower limit for a time.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub minimum: Option<Time>,

    /// The inclusive upper limit for a time.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub maximum: Option<Time>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl TimeValidator {
    const NICK: [u8; 3] = *b"tmv";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::TimeValidator
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
