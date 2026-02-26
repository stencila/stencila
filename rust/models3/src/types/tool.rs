use serde::{Deserialize, Serialize};
use serde_json::de::Deserializer as JsonDeserializer;

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

impl ToolCall {
    /// Parse a raw argument string into structured JSON.
    ///
    /// Returns `(parsed_value, parse_error)`. If the raw string is empty,
    /// returns an empty object. If parsing fails, attempts to salvage a
    /// JSON object from the string by scanning for the first `{` — some
    /// reasoning models (e.g. GPT-5.x-codex) occasionally prepend garbled
    /// reasoning tokens to the arguments string.
    #[must_use]
    pub fn parse_arguments(raw: &str) -> (serde_json::Value, Option<String>) {
        if raw.is_empty() {
            return (serde_json::Value::Object(serde_json::Map::new()), None);
        }

        match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(parsed) => (parsed, None),
            Err(original_err) => {
                // Attempt to salvage: find the first '{' and try parsing
                // from there. This handles reasoning tokens prepended to
                // the JSON arguments by some models.
                if let Some(salvaged) = salvage_json_object(raw) {
                    return (salvaged, None);
                }
                (
                    serde_json::Value::String(raw.to_string()),
                    Some(original_err.to_string()),
                )
            }
        }
    }
}

/// Try to extract a valid JSON object from a string that may have
/// non-JSON text prepended (e.g. reasoning tokens from the model).
///
/// Scans forward from the first `{` and attempts to parse progressively
/// longer substrings ending at each `}` from the end. Returns `Some`
/// on the first successful parse, `None` if no valid object is found.
fn salvage_json_object(raw: &str) -> Option<serde_json::Value> {
    // Try each `{` as a candidate start so earlier brace-like noise does not
    // block recovery of a valid JSON object later in the string.
    for (start, _) in raw.match_indices('{') {
        let candidate = &raw[start..];
        let mut de = JsonDeserializer::from_str(candidate);
        if let Ok(parsed) = serde_json::Value::deserialize(&mut de)
            && parsed.is_object()
        {
            return Some(parsed);
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_arguments_empty() {
        let (val, err) = ToolCall::parse_arguments("");
        assert!(err.is_none());
        assert!(val.is_object());
        assert!(val.as_object().expect("should be object").is_empty());
    }

    #[test]
    fn parse_arguments_valid_json() {
        let (val, err) = ToolCall::parse_arguments(r#"{"command":"ls"}"#);
        assert!(err.is_none());
        assert_eq!(val["command"], "ls");
    }

    #[test]
    fn parse_arguments_garbled_prefix() {
        let raw = r#"Running tests numerusformassistant to=functions.shell json{"command":"cargo test","timeout_ms":120000}"#;
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_none(), "expected salvaged parse, got: {err:?}");
        assert_eq!(val["command"], "cargo test");
        assert_eq!(val["timeout_ms"], 120_000);
    }

    #[test]
    fn parse_arguments_garbled_prefix_with_unicode() {
        let raw = "+#+#+#+#+#+assistant to=functions.shell մեկնաdelays 天天中彩票json{\"file_path\":\"/tmp/test.rs\"}";
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_none(), "expected salvaged parse, got: {err:?}");
        assert_eq!(val["file_path"], "/tmp/test.rs");
    }

    #[test]
    fn parse_arguments_garbled_prefix_and_suffix() {
        let raw = r#"some garbage {"key":"value"} trailing text"#;
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_none(), "expected salvaged parse, got: {err:?}");
        assert_eq!(val["key"], "value");
    }

    #[test]
    fn parse_arguments_skips_earlier_non_json_braces() {
        let raw = r#"preface {not-json} assistant to=functions.shell json {"command":"cargo test","timeout_ms":120000}"#;
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_none(), "expected salvaged parse, got: {err:?}");
        assert_eq!(val["command"], "cargo test");
        assert_eq!(val["timeout_ms"], 120_000);
    }

    #[test]
    fn parse_arguments_assistant_wrapper_then_json_newline() {
        let raw = "Final commit verification assistant to=functions.shell commentary \u{10d8}\u{10ec}\u{10dc} json\n\n{\"command\":\"git log -1 --pretty=fuller\",\"timeout_ms\":10000}";
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_none(), "expected salvaged parse, got: {err:?}");
        assert_eq!(val["command"], "git log -1 --pretty=fuller");
        assert_eq!(val["timeout_ms"], 10_000);
    }

    #[test]
    fn parse_arguments_no_json_at_all() {
        let (val, err) = ToolCall::parse_arguments("just plain text with no braces");
        assert!(err.is_some());
        assert_eq!(
            val.as_str().expect("should be string"),
            "just plain text with no braces"
        );
    }

    #[test]
    fn parse_arguments_no_valid_json_in_braces() {
        let (val, err) = ToolCall::parse_arguments("prefix {not valid json} suffix");
        assert!(err.is_some());
        assert!(val.is_string());
    }

    #[test]
    fn salvage_nested_braces() {
        let raw = r#"garble {"outer":{"inner":"value"}}"#;
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_none(), "expected salvaged parse, got: {err:?}");
        assert_eq!(val["outer"]["inner"], "value");
    }

    #[test]
    fn salvage_does_not_return_arrays() {
        let raw = "prefix [1,2,3]";
        let (val, err) = ToolCall::parse_arguments(raw);
        assert!(err.is_some(), "arrays should not be salvaged as tool args");
        assert!(val.is_string());
    }
}
