// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// A schema specifying constraints on a string node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("StringValidator")]
pub struct StringValidator {
    /// The type of this item.
    pub r#type: MustBe!("StringValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The minimum length for a string node.
    #[serde(alias = "min-length", alias = "min_length")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub min_length: Option<Integer>,

    /// The maximum length for a string node.
    #[serde(alias = "max-length", alias = "max_length")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub max_length: Option<Integer>,

    /// A regular expression that a string node must match.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub pattern: Option<String>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl StringValidator {
    const NICK: [u8; 3] = [115, 116, 118];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::StringValidator
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
