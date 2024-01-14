// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A variable representing a name / value pair.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Variable")]
pub struct Variable {
    /// The type of this item.
    pub r#type: MustBe!("Variable"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the variable.
    pub name: String,

    /// The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`
    pub kind: Option<String>,

    /// The value of the variable.
    pub value: Option<Box<Node>>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Variable {
    const NICK: &'static str = "var";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Variable
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
