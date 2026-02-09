use serde::{Deserialize, Serialize};

/// The role of a message participant in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// High-level instructions shaping model behavior.
    System,
    /// Human input (text, images, audio, documents).
    User,
    /// Model output (text, tool calls, thinking blocks).
    Assistant,
    /// Tool execution results (linked by `tool_call_id`).
    Tool,
    /// Privileged instructions from the application (not end user).
    Developer,
}
