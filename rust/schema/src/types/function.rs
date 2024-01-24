// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::parameter::Parameter;
use super::string::String;
use super::validator::Validator;

/// A function with a name, which might take Parameters and return a value of a certain type.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Function")]
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

    /// Non-core optional fields
    #[serde(flatten)]
    #[dom(elem = "none")]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<FunctionOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct FunctionOptions {
    /// The return type of the function.
    pub returns: Option<Validator>,
}

impl Function {
    const NICK: &'static str = "fun";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Function
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, parameters: Vec<Parameter>) -> Self {
        Self {
            name,
            parameters,
            ..Default::default()
        }
    }
}
