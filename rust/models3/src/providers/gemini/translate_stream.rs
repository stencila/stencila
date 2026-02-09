use serde_json::Value;

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::sse::SseEvent;
use crate::providers::gemini::translate_error::translate_error;
use crate::providers::gemini::translate_response::{map_finish_reason, parse_usage};
use crate::types::content::{ContentPart, ThinkingData, ToolCallData};
use crate::types::finish_reason::FinishReason;
use crate::types::message::Message;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::ToolCall;

/// Stateful translator for Gemini SSE events.
#[derive(Debug, Clone, Default)]
pub struct GeminiStreamState {
    emitted_stream_start: bool,
    text_started: bool,
    finished: bool,
    finish_reason: Option<String>,
    usage: Option<Value>,
    rate_limit: Option<RateLimitInfo>,
    /// Accumulated content parts for building the full response on FINISH.
    accumulated_content: Vec<ContentPart>,
    /// Accumulated text for the current text segment.
    accumulated_text: String,
    /// Accumulated thinking/reasoning text.
    accumulated_thinking: String,
    /// Model version from SSE payload (if available).
    model: Option<String>,
}

impl GeminiStreamState {
    #[must_use]
    pub fn with_rate_limit(rate_limit: Option<RateLimitInfo>) -> Self {
        Self {
            rate_limit,
            ..Self::default()
        }
    }
}

/// Translate a single parsed SSE event into one or more unified stream events.
///
/// # Errors
///
/// Returns `SdkError::Stream` if the SSE payload is not valid JSON.
pub fn translate_sse_event(
    event: &SseEvent,
    state: &mut GeminiStreamState,
) -> SdkResult<Vec<StreamEvent>> {
    let mut out = Vec::new();

    let payload: Value = serde_json::from_str(&event.data).map_err(|e| SdkError::Stream {
        message: format!("invalid Gemini SSE payload JSON: {e}"),
    })?;

    // Check for error responses in the stream
    if let Some(error) = payload.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Gemini stream error")
            .to_string();

        let err = translate_error(SdkError::Server {
            message,
            details: ProviderDetails {
                provider: Some("gemini".to_string()),
                raw: Some(payload.clone()),
                retryable: true,
                ..ProviderDetails::default()
            },
        });
        state.finished = true;
        out.push(StreamEvent::error(err));
        return Ok(out);
    }

    // Emit stream start on first chunk
    if !state.emitted_stream_start {
        state.emitted_stream_start = true;
        out.push(StreamEvent {
            event_type: StreamEventType::StreamStart,
            delta: None,
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: None,
            warnings: None,
            raw: None,
        });
    }

    // Store usage metadata if present
    if let Some(usage_metadata) = payload.get("usageMetadata") {
        state.usage = Some(usage_metadata.clone());
    }

    // Capture model version if present
    if state.model.is_none()
        && let Some(model) = payload.get("modelVersion").and_then(Value::as_str)
    {
        state.model = Some(model.to_string());
    }

    // Process candidates
    if let Some(candidates) = payload.get("candidates").and_then(Value::as_array)
        && let Some(candidate) = candidates.first()
    {
        // Process content parts before checking finish reason, since the
        // chunk with finishReason may also carry the final text delta.
        if let Some(parts) = candidate
            .pointer("/content/parts")
            .and_then(Value::as_array)
        {
            for part in parts {
                translate_stream_part(part, state, &mut out, &payload);
            }
        }

        // When finishReason arrives, emit TEXT_END immediately rather than
        // deferring to on_stream_end. This gives consumers timely lifecycle
        // events per spec Section 7.7.
        if let Some(finish_reason) = candidate.get("finishReason").and_then(Value::as_str) {
            state.finish_reason = Some(finish_reason.to_string());

            if state.text_started {
                state.text_started = false;
                out.push(StreamEvent {
                    event_type: StreamEventType::TextEnd,
                    delta: None,
                    text_id: Some("text_0".to_string()),
                    reasoning_delta: None,
                    tool_call: None,
                    finish_reason: None,
                    usage: None,
                    response: None,
                    error: None,
                    warnings: None,
                    raw: Some(payload.clone()),
                });
            }
        }
    }

    Ok(out)
}

fn translate_stream_part(
    part: &Value,
    state: &mut GeminiStreamState,
    out: &mut Vec<StreamEvent>,
    raw: &Value,
) {
    if let Some(text) = part.get("text").and_then(Value::as_str) {
        translate_text_part(text, part, state, out, raw);
    } else if let Some(function_call) = part.get("functionCall") {
        translate_function_call_part(function_call, state, out, raw);
    }
}

fn translate_text_part(
    text: &str,
    part: &Value,
    state: &mut GeminiStreamState,
    out: &mut Vec<StreamEvent>,
    raw: &Value,
) {
    let is_thought = part
        .get("thought")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if is_thought {
        state.accumulated_thinking.push_str(text);
        out.push(StreamEvent {
            event_type: StreamEventType::ReasoningDelta,
            delta: None,
            text_id: None,
            reasoning_delta: Some(text.to_string()),
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: None,
            warnings: None,
            raw: Some(raw.clone()),
        });
    } else {
        state.accumulated_text.push_str(text);
        if !state.text_started {
            state.text_started = true;
            out.push(StreamEvent {
                event_type: StreamEventType::TextStart,
                delta: None,
                text_id: Some("text_0".to_string()),
                reasoning_delta: None,
                tool_call: None,
                finish_reason: None,
                usage: None,
                response: None,
                error: None,
                warnings: None,
                raw: Some(raw.clone()),
            });
        }

        out.push(StreamEvent {
            event_type: StreamEventType::TextDelta,
            delta: Some(text.to_string()),
            text_id: Some("text_0".to_string()),
            reasoning_delta: None,
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: None,
            warnings: None,
            raw: Some(raw.clone()),
        });
    }
}

fn translate_function_call_part(
    function_call: &Value,
    state: &mut GeminiStreamState,
    out: &mut Vec<StreamEvent>,
    raw: &Value,
) {
    let name = function_call
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unknown_function")
        .to_string();

    let args = function_call
        .get("args")
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));

    let raw_args = serde_json::to_string(&args).ok();
    let id = format!("call_{}", uuid::Uuid::new_v4());

    // Accumulate tool call for the full response
    state.accumulated_content.push(ContentPart::ToolCall {
        tool_call: ToolCallData {
            id: id.clone(),
            name: name.clone(),
            arguments: args.clone(),
            call_type: "function".to_string(),
        },
    });

    out.push(StreamEvent {
        event_type: StreamEventType::ToolCallStart,
        delta: None,
        text_id: None,
        reasoning_delta: None,
        tool_call: Some(ToolCall {
            id: id.clone(),
            name: name.clone(),
            arguments: Value::String(String::new()),
            raw_arguments: Some(String::new()),
            parse_error: None,
        }),
        finish_reason: None,
        usage: None,
        response: None,
        error: None,
        warnings: None,
        raw: Some(raw.clone()),
    });

    out.push(StreamEvent {
        event_type: StreamEventType::ToolCallEnd,
        delta: None,
        text_id: None,
        reasoning_delta: None,
        tool_call: Some(ToolCall {
            id,
            name,
            arguments: args,
            raw_arguments: raw_args,
            parse_error: None,
        }),
        finish_reason: None,
        usage: None,
        response: None,
        error: None,
        warnings: None,
        raw: Some(raw.clone()),
    });
}

impl crate::providers::common::stream::SseStreamState for GeminiStreamState {
    fn translate_event(&mut self, event: &SseEvent) -> SdkResult<Vec<StreamEvent>> {
        translate_sse_event(event, self)
    }

    #[allow(clippy::too_many_lines)]
    fn on_stream_end(&mut self) -> Vec<StreamEvent> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;

        let mut out = Vec::new();

        // Empty stream: emit stream start before finish
        if !self.emitted_stream_start {
            self.emitted_stream_start = true;
            out.push(StreamEvent::stream_start());
        }

        if self.text_started {
            // Accumulate remaining text content
            let text = std::mem::take(&mut self.accumulated_text);
            if !text.is_empty() {
                self.accumulated_content.push(ContentPart::Text { text });
            }
            out.push(StreamEvent {
                event_type: StreamEventType::TextEnd,
                delta: None,
                text_id: Some("text_0".to_string()),
                reasoning_delta: None,
                tool_call: None,
                finish_reason: None,
                usage: None,
                response: None,
                error: None,
                warnings: None,
                raw: None,
            });
            self.text_started = false;
        }

        // Accumulate thinking content if present
        let thinking = std::mem::take(&mut self.accumulated_thinking);
        if !thinking.is_empty() {
            self.accumulated_content.push(ContentPart::Thinking {
                thinking: ThinkingData {
                    text: thinking,
                    signature: None,
                    redacted: false,
                },
            });
        }

        let usage = parse_usage(self.usage.as_ref());
        let content = std::mem::take(&mut self.accumulated_content);

        let reason = if content
            .iter()
            .any(|part| matches!(part, ContentPart::ToolCall { .. }))
        {
            crate::types::finish_reason::Reason::ToolCalls
        } else {
            map_finish_reason(self.finish_reason.as_deref())
        };

        let finish_reason = FinishReason {
            reason,
            raw: self.finish_reason.take(),
        };

        let id = format!("gemini_{}", uuid::Uuid::new_v4());
        let model = self.model.clone().unwrap_or_else(|| "unknown".to_string());

        let response = Response {
            id,
            model,
            provider: "gemini".to_string(),
            message: Message {
                role: Role::Assistant,
                content,
                name: None,
                tool_call_id: None,
            },
            finish_reason: finish_reason.clone(),
            usage: usage.clone(),
            raw: None,
            warnings: None,
            rate_limit: self.rate_limit.clone(),
        };

        out.push(StreamEvent {
            event_type: StreamEventType::Finish,
            delta: None,
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: Some(finish_reason),
            usage: Some(usage),
            response: Some(Box::new(response)),
            error: None,
            warnings: None,
            raw: None,
        });

        out
    }
}

/// Translate a full SSE stream into unified stream events.
#[must_use]
pub fn translate_sse_stream<'a>(
    sse_stream: std::pin::Pin<Box<dyn futures::Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
    rate_limit: Option<RateLimitInfo>,
) -> crate::provider::BoxStream<'a, SdkResult<StreamEvent>> {
    crate::providers::common::stream::translate_sse_stream(
        sse_stream,
        GeminiStreamState::with_rate_limit(rate_limit),
    )
}
