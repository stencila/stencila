// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::integer::Integer;
use super::string::String;
use super::validator::Validator;

/// A validator specifying constraints on an array node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("ArrayValidator")]
pub struct ArrayValidator {
    /// The type of this item.
    pub r#type: MustBe!("ArrayValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Whether items can have the value `Node::Null`
    #[serde(alias = "items-nullable", alias = "items_nullable")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub items_nullable: Option<Boolean>,

    /// Another validator node specifying the constraints on all items in the array.
    #[serde(alias = "items-validator", alias = "items_validator")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub items_validator: Option<Box<Validator>>,

    /// An array node is valid if at least one of its items is valid against the `contains` schema.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub contains: Option<Box<Validator>>,

    /// An array node is valid if its size is greater than, or equal to, this value.
    #[serde(alias = "min-items", alias = "min_items")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub min_items: Option<Integer>,

    /// An array node is valid if its size is less than, or equal to, this value.
    #[serde(alias = "max-items", alias = "max_items")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub max_items: Option<Integer>,

    /// A flag to indicate that each value in the array should be unique.
    #[serde(alias = "unique-items", alias = "unique_items")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub unique_items: Option<Boolean>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ArrayValidator {
    const NICK: [u8; 3] = *b"arv";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ArrayValidator
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
