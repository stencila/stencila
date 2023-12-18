// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::code_location::CodeLocation;
use super::string::String;

/// An error that occurred while compiling an executable node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CompilationError")]
pub struct CompilationError {
    /// The type of this item.
    pub r#type: MustBe!("CompilationError"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The error message or brief description of the error.
    #[serde(alias = "message", alias = "error-message", alias = "error_message")]
    pub error_message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    #[serde(alias = "error-type", alias = "error_type")]
    pub error_type: Option<String>,

    /// The location that the error occurred.
    #[serde(alias = "code-location", alias = "code_location")]
    pub code_location: Option<CodeLocation>,

    /// A unique identifier for this node
    
    #[serde(skip)]
    pub node_id: NodeId
}

impl CompilationError {
    pub fn new(error_message: String) -> Self {
        Self {
            error_message,
            ..Default::default()
        }
    }
}

impl Entity for CompilationError {
    fn node_type() -> NodeType {
        NodeType::CompilationError
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}
