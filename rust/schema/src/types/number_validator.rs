// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// A validator specifying the constraints on a numeric node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "NumberValidator")]
pub struct NumberValidator {
    /// The type of this item.
    pub r#type: MustBe!("NumberValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The inclusive lower limit for a numeric node.
    #[patch(format = "md", format = "myst")]
    pub minimum: Option<Number>,

    /// The exclusive lower limit for a numeric node.
    #[serde(alias = "exclusive-minimum", alias = "exclusive_minimum")]
    #[patch(format = "md", format = "myst")]
    pub exclusive_minimum: Option<Number>,

    /// The inclusive upper limit for a numeric node.
    #[patch(format = "md", format = "myst")]
    pub maximum: Option<Number>,

    /// The exclusive upper limit for a numeric node.
    #[serde(alias = "exclusive-maximum", alias = "exclusive_maximum")]
    #[patch(format = "md", format = "myst")]
    pub exclusive_maximum: Option<Number>,

    /// A number that a numeric node must be a multiple of.
    #[serde(alias = "multiple-of", alias = "multiple_of")]
    #[patch(format = "md", format = "myst")]
    pub multiple_of: Option<Number>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl NumberValidator {
    const NICK: [u8; 3] = [110, 117, 118];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::NumberValidator
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
