// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::code_location::CodeLocation;
use super::message_level::MessageLevel;
use super::string::String;

/// An error, warning or log message generated during compilation.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CompilationMessage")]
pub struct CompilationMessage {
    /// The type of this item.
    pub r#type: MustBe!("CompilationMessage"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The severity level of the message.
    pub level: MessageLevel,

    /// The text of the message.
    pub message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    #[serde(alias = "error-type", alias = "error_type")]
    pub error_type: Option<String>,

    /// The location that the error occurred.
    #[serde(alias = "code-location", alias = "code_location")]
    pub code_location: Option<CodeLocation>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl CompilationMessage {
    const NICK: [u8; 3] = [99, 109, 101];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::CompilationMessage
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
