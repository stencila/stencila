// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::date::Date;
use super::string::String;

/// A validator specifying the constraints on a date.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("DateValidator")]
pub struct DateValidator {
    /// The type of this item.
    pub r#type: MustBe!("DateValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The inclusive lower limit for a date.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub minimum: Option<Date>,

    /// The inclusive upper limit for a date.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub maximum: Option<Date>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl DateValidator {
    const NICK: [u8; 3] = *b"dav";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::DateValidator
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
