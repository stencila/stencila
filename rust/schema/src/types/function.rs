// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::parameter::Parameter;
use super::string::String;
use super::validator::Validator;

/// A function with a name, which might take Parameters and return a value of a certain type.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("Function")]
pub struct Function {
    /// The type of this item.
    pub r#type: MustBe!("Function"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the function.
    pub name: String,

    /// The parameters of the function.
    #[serde(alias = "parameter")]
    #[serde(deserialize_with = "one_or_many")]
    pub parameters: Vec<Parameter>,

    /// The return type of the function.
    pub returns: Option<Validator>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Function {
    const NICK: [u8; 3] = [102, 117, 110];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Function
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, parameters: Vec<Parameter>) -> Self {
        Self {
            name,
            parameters,
            ..Default::default()
        }
    }
}
