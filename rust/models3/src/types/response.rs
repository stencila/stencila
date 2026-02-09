use serde::{Deserialize, Serialize};

use super::content::ContentPart;
use super::finish_reason::FinishReason;
use super::message::Message;
use super::rate_limit::RateLimitInfo;
use super::tool::ToolCall;
use super::usage::Usage;
use super::warning::Warning;

/// A unified response from an LLM provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Provider-assigned response ID.
    pub id: String,
    /// The model that actually generated the response.
    pub model: String,
    /// Which provider fulfilled the request.
    pub provider: String,
    /// The assistant's response message.
    pub message: Message,
    /// Why the model stopped generating.
    pub finish_reason: FinishReason,
    /// Token usage information.
    pub usage: Usage,
    /// Raw provider response JSON for debugging.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
    /// Non-fatal warnings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
    /// Rate limit information from response headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitInfo>,
}

impl Response {
    /// Concatenate all text content parts from the response message.
    #[must_use]
    pub fn text(&self) -> String {
        self.message.text()
    }

    /// Extract all tool calls from the response message as unified `ToolCall` values.
    ///
    /// Converts from content-level `ToolCallData` (which may hold unparsed arguments)
    /// to the unified `ToolCall` type (always-parsed arguments + optional raw string).
    #[must_use]
    pub fn tool_calls(&self) -> Vec<ToolCall> {
        self.message
            .content
            .iter()
            .filter_map(|part| match part {
                ContentPart::ToolCall { tool_call } => {
                    let (arguments, raw_arguments, parse_error) =
                        if let Some(raw) = tool_call.arguments.as_str() {
                            match serde_json::from_str::<serde_json::Value>(raw) {
                                Ok(parsed) => (parsed, Some(raw.to_string()), None),
                                Err(e) => (
                                    serde_json::Value::String(raw.to_string()),
                                    Some(raw.to_string()),
                                    Some(e.to_string()),
                                ),
                            }
                        } else {
                            (tool_call.arguments.clone(), None, None)
                        };
                    Some(ToolCall {
                        id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        arguments,
                        raw_arguments,
                        parse_error,
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// Concatenate all reasoning/thinking text from the response message.
    ///
    /// Returns `None` if no thinking parts are present, distinguishing
    /// "no reasoning" from "empty reasoning text".
    #[must_use]
    pub fn reasoning(&self) -> Option<String> {
        let parts: Vec<&str> = self
            .message
            .content
            .iter()
            .filter_map(|part| match part {
                ContentPart::Thinking { thinking } => Some(thinking.text.as_str()),
                _ => None,
            })
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(""))
        }
    }
}
