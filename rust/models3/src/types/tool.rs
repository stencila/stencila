use serde::{Deserialize, Serialize};

use crate::error::{ProviderDetails, SdkError, SdkResult};

/// Definition of a tool that the model can call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The tool's name: `[a-zA-Z][a-zA-Z0-9_]*`, max 64 characters.
    pub name: String,
    /// Human-readable description of what the tool does (required by spec).
    pub description: String,
    /// JSON Schema describing the tool's parameters.
    /// Root must be `"type": "object"` per spec.
    pub parameters: serde_json::Value,
    /// Whether to enforce strict schema validation.
    #[serde(default)]
    pub strict: bool,
}

impl ToolDefinition {
    /// Validate the tool definition against spec constraints.
    ///
    /// Checks:
    /// - Name matches `[a-zA-Z][a-zA-Z0-9_]*` and is at most 64 characters.
    /// - Description is non-empty.
    /// - Parameters root has `"type": "object"`.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::InvalidRequest` if any constraint is violated.
    pub fn validate(&self) -> SdkResult<()> {
        // Name: must be a valid identifier, max 64 chars
        if self.name.is_empty() || self.name.len() > 64 {
            return Err(SdkError::InvalidRequest {
                message: format!(
                    "tool name must be 1-64 characters, got {} characters",
                    self.name.len()
                ),
                details: ProviderDetails::default(),
            });
        }
        let mut chars = self.name.chars();
        let first = chars.next(); // safe: name is non-empty
        if !first.is_some_and(|c| c.is_ascii_alphabetic()) {
            return Err(SdkError::InvalidRequest {
                message: format!("tool name must start with a letter, got {:?}", self.name),
                details: ProviderDetails::default(),
            });
        }
        if let Some(bad) = chars.find(|c| !c.is_ascii_alphanumeric() && *c != '_') {
            return Err(SdkError::InvalidRequest {
                message: format!(
                    "tool name contains invalid character '{bad}' in {:?}",
                    self.name
                ),
                details: ProviderDetails::default(),
            });
        }

        // Description must be non-empty
        if self.description.is_empty() {
            return Err(SdkError::InvalidRequest {
                message: "tool description must be non-empty".into(),
                details: ProviderDetails::default(),
            });
        }

        // Parameters root must be "type": "object"
        if self.parameters.get("type").and_then(|v| v.as_str()) != Some("object") {
            return Err(SdkError::InvalidRequest {
                message: "tool parameters must have root \"type\": \"object\"".into(),
                details: ProviderDetails::default(),
            });
        }

        Ok(())
    }
}

/// A unified tool call extracted from a response.
///
/// Unlike `ToolCallData` (which lives in message content and may have unparsed
/// arguments), `ToolCall` always has parsed arguments and optionally preserves
/// the raw argument string for debugging.
///
/// When the provider returns argument strings that are not valid JSON,
/// `arguments` is set to the raw string as `Value::String` and `parse_error`
/// describes the failure. Callers should check `parse_error` before using
/// `arguments` as a JSON object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier (provider-assigned).
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Parsed JSON arguments, or the raw string if parsing failed.
    pub arguments: serde_json::Value,
    /// Raw argument string before parsing, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_arguments: Option<String>,
    /// If the raw argument string could not be parsed as JSON, this
    /// contains the parse error message. Callers should treat the tool
    /// call as invalid when this is `Some`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

/// A unified tool result returned from tool execution.
///
/// Unlike `ToolResultData` (which lives in message content and carries
/// optional image data for round-tripping), `ToolResult` is the high-level
/// type used in the tool execution loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Correlates to `ToolCall.id`.
    pub tool_call_id: String,
    /// The tool's output (string, dict, or list).
    pub content: serde_json::Value,
    /// True if the tool execution failed.
    #[serde(default)]
    pub is_error: bool,
}

/// Controls how the model uses tools.
///
/// The spec defines this as `{ type: string, tool_name?: string }`, but a
/// Rust enum is more idiomatic: it makes invalid states unrepresentable
/// (e.g. `tool_name` set on a non-`tool` type) and enables exhaustive
/// pattern matching. Provider adapters translate to/from the wire format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    /// Model decides whether to call tools.
    Auto,
    /// Model must not call any tools.
    None,
    /// Model must call at least one tool.
    Required,
    /// Model must call this specific tool.
    Tool(String),
}
