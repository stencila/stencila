use serde::{Deserialize, Serialize};

use super::content::ContentPart;
use super::role::Role;

/// A single message in the conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The multimodal content parts.
    pub content: Vec<ContentPart>,
    /// Optional name (for tool messages and developer attribution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Links a tool-result message to its originating tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a system message from text.
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a user message from text.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    /// Create an assistant message from text.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a developer message from text.
    pub fn developer(text: impl Into<String>) -> Self {
        Self {
            role: Role::Developer,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a tool result message with structured content.
    pub fn tool_result(
        tool_call_id: impl Into<String>,
        content: serde_json::Value,
        is_error: bool,
    ) -> Self {
        let id = tool_call_id.into();
        Self {
            role: Role::Tool,
            content: vec![ContentPart::tool_result(&id, content, is_error)],
            name: None,
            tool_call_id: Some(id),
        }
    }

    /// Create a tool result message with plain text content.
    pub fn tool_result_text(
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self::tool_result(
            tool_call_id,
            serde_json::Value::String(content.into()),
            is_error,
        )
    }

    /// Create a message with specific role and content parts.
    #[must_use]
    pub fn new(role: Role, content: Vec<ContentPart>) -> Self {
        Self {
            role,
            content,
            name: None,
            tool_call_id: None,
        }
    }

    /// Concatenate all text content parts into a single string.
    ///
    /// Returns an empty string if there are no text parts.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(ContentPart::as_text)
            .collect::<Vec<_>>()
            .join("")
    }
}
