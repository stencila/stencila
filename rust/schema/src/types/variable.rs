// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::hint::Hint;
use super::node::Node;
use super::string::String;

/// A variable representing a name / value pair.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Variable")]
pub struct Variable {
    /// The type of this item.
    pub r#type: MustBe!("Variable"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The name of the variable.
    pub name: String,

    /// The programming language that the variable is defined in e.g. Python, JSON.
    #[serde(alias = "programming-language", alias = "programming_language")]
    pub programming_language: Option<String>,

    /// The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame`
    #[serde(alias = "native-type", alias = "native_type")]
    pub native_type: Option<String>,

    /// The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`.
    #[serde(alias = "node-type", alias = "node_type")]
    pub node_type: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<VariableOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct VariableOptions {
    /// The value of the variable.
    pub value: Option<Node>,

    /// A hint to the value and/or structure of the variable.
    pub hint: Option<Hint>,

    /// A textual hint to the value and/or structure of the variable.
    #[serde(alias = "native-hint", alias = "native_hint")]
    pub native_hint: Option<String>,
}

impl Variable {
    const NICK: [u8; 3] = *b"var";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Variable
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
