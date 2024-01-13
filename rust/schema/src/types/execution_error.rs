// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::code_location::CodeLocation;
use super::string::String;

/// An error that occurred when executing an executable node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ExecutionError")]
pub struct ExecutionError {
    /// The type of this item.
    pub r#type: MustBe!("ExecutionError"),

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

    /// Stack trace leading up to the error.
    #[serde(alias = "trace", alias = "stack-trace", alias = "stack_trace")]
    pub stack_trace: Option<String>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ExecutionError {
    const NICK: &'static str = "exe";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ExecutionError
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(error_message: String) -> Self {
        Self {
            error_message,
            ..Default::default()
        }
    }
}
