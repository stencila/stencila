// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::code_location::CodeLocation;
use super::message_level::MessageLevel;
use super::string::String;

/// An error, warning or log message generated during execution.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ExecutionMessage")]
pub struct ExecutionMessage {
    /// The type of this item.
    pub r#type: MustBe!("ExecutionMessage"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The severity level of the message.
    pub level: MessageLevel,

    /// The text of the message.
    #[dom(elem = "pre")]
    pub message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    #[serde(alias = "error-type", alias = "error_type")]
    pub error_type: Option<String>,

    /// The location that the error occurred or other message emanated from.
    #[serde(alias = "code-location", alias = "code_location")]
    #[dom(elem = "div")]
    pub code_location: Option<CodeLocation>,

    /// Stack trace leading up to the error.
    #[serde(alias = "trace", alias = "stack-trace", alias = "stack_trace")]
    #[dom(elem = "pre")]
    pub stack_trace: Option<String>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ExecutionMessage {
    const NICK: [u8; 3] = [101, 109, 101];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ExecutionMessage
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(level: MessageLevel, message: String) -> Self {
        Self {
            level,
            message,
            ..Default::default()
        }
    }
}
