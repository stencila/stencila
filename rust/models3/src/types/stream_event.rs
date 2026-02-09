use serde::{Deserialize, Serialize};

use crate::error::SdkError;

use super::finish_reason::FinishReason;
use super::tool::ToolCall;
use super::usage::Usage;
use super::warning::Warning;

/// A single event in a streaming response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamEvent {
    /// The event type discriminator.
    #[serde(rename = "type")]
    pub event_type: StreamEventType,
    /// Incremental text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    /// Identifies the text segment this delta belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_id: Option<String>,
    /// Incremental reasoning/thinking text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_delta: Option<String>,
    /// Partial or complete tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCall>,
    /// Why generation stopped (present on FINISH events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// Token usage (present on FINISH events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Full accumulated response (present on FINISH events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Box<super::response::Response>>,
    /// Error payload (present on ERROR events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SdkError>,
    /// Warnings (present on `STREAM_START` events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
    /// Raw provider event for passthrough.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// The type of a stream event.
///
/// Known event types have explicit variants. Unknown event type strings
/// (from future spec versions or provider-specific events) are captured
/// by the `Unknown` variant for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamEventType {
    /// Stream has begun.
    StreamStart,
    /// New text segment begun.
    TextStart,
    /// Incremental text content.
    TextDelta,
    /// Text segment complete.
    TextEnd,
    /// Model reasoning has begun.
    ReasoningStart,
    /// Incremental reasoning content.
    ReasoningDelta,
    /// Reasoning complete.
    ReasoningEnd,
    /// Tool call has begun.
    ToolCallStart,
    /// Incremental tool call arguments.
    ToolCallDelta,
    /// Tool call fully formed.
    ToolCallEnd,
    /// Generation complete.
    Finish,
    /// Error during streaming.
    Error,
    /// Raw provider event not mapped to unified model.
    ProviderEvent,
    /// Unknown event type string (forward compatibility).
    Unknown(String),
}

impl Serialize for StreamEventType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Self::StreamStart => "stream_start",
            Self::TextStart => "text_start",
            Self::TextDelta => "text_delta",
            Self::TextEnd => "text_end",
            Self::ReasoningStart => "reasoning_start",
            Self::ReasoningDelta => "reasoning_delta",
            Self::ReasoningEnd => "reasoning_end",
            Self::ToolCallStart => "tool_call_start",
            Self::ToolCallDelta => "tool_call_delta",
            Self::ToolCallEnd => "tool_call_end",
            Self::Finish => "finish",
            Self::Error => "error",
            Self::ProviderEvent => "provider_event",
            Self::Unknown(s) => s.as_str(),
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for StreamEventType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "stream_start" => Self::StreamStart,
            "text_start" => Self::TextStart,
            "text_delta" => Self::TextDelta,
            "text_end" => Self::TextEnd,
            "reasoning_start" => Self::ReasoningStart,
            "reasoning_delta" => Self::ReasoningDelta,
            "reasoning_end" => Self::ReasoningEnd,
            "tool_call_start" => Self::ToolCallStart,
            "tool_call_delta" => Self::ToolCallDelta,
            "tool_call_end" => Self::ToolCallEnd,
            "finish" => Self::Finish,
            "error" => Self::Error,
            "provider_event" => Self::ProviderEvent,
            _ => Self::Unknown(s),
        })
    }
}

impl StreamEvent {
    /// Create a text delta event.
    pub fn text_delta(delta: impl Into<String>) -> Self {
        Self {
            event_type: StreamEventType::TextDelta,
            delta: Some(delta.into()),
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: None,
            warnings: None,
            raw: None,
        }
    }

    /// Create a finish event.
    #[must_use]
    pub fn finish(finish_reason: FinishReason, usage: Usage) -> Self {
        Self {
            event_type: StreamEventType::Finish,
            delta: None,
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: Some(finish_reason),
            usage: Some(usage),
            response: None,
            error: None,
            warnings: None,
            raw: None,
        }
    }

    /// Create an error event.
    #[must_use]
    pub fn error(error: SdkError) -> Self {
        Self {
            event_type: StreamEventType::Error,
            delta: None,
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: Some(error),
            warnings: None,
            raw: None,
        }
    }
}
