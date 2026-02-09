use std::collections::{BTreeMap, HashSet};

use serde_json::Value;

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::sse::SseEvent;
use crate::providers::openai_chat_completions::translate_error::translate_error;
use crate::providers::openai_chat_completions::translate_response::{
    map_finish_reason, parse_usage,
};
use crate::types::content::{ContentPart, ToolCallData};
use crate::types::finish_reason::FinishReason;
use crate::types::message::Message;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::ToolCall;
use crate::types::usage::Usage;

#[derive(Debug, Clone, Default)]
struct ToolCallState {
    id: String,
    name: String,
    arguments: String,
    started: bool,
}

/// Stateful translator for OpenAI-compatible Chat Completions SSE events.
#[derive(Debug, Clone, Default)]
pub struct OpenAIChatCompletionsStreamState {
    emitted_stream_start: bool,
    emitted_text_start: bool,
    tool_calls: BTreeMap<u64, ToolCallState>,
    pending_usage: Option<Usage>,
    finished: bool,
    seen_finish_text_ids: HashSet<String>,
    /// Rate limit info from response headers.
    rate_limit: Option<RateLimitInfo>,
    /// Accumulated text for building the full response.
    accumulated_text: String,
    /// Response ID from SSE payload.
    response_id: Option<String>,
    /// Model from SSE payload.
    response_model: Option<String>,
}

impl OpenAIChatCompletionsStreamState {
    #[must_use]
    pub fn with_rate_limit(rate_limit: Option<RateLimitInfo>) -> Self {
        Self {
            rate_limit,
            ..Self::default()
        }
    }
}

/// Translate a single SSE event into zero or more unified stream events.
///
/// # Errors
///
/// Returns `SdkError::Stream` when provider event payloads are malformed JSON.
#[allow(clippy::too_many_lines)]
pub fn translate_sse_event(
    event: &SseEvent,
    state: &mut OpenAIChatCompletionsStreamState,
) -> SdkResult<Vec<StreamEvent>> {
    let mut out = Vec::new();

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

    let payload: Value = serde_json::from_str(&event.data).map_err(|e| SdkError::Stream {
        message: format!("invalid Chat Completions SSE payload: {e}"),
    })?;

    // Capture response ID and model from the payload
    if state.response_id.is_none()
        && let Some(id) = payload.get("id").and_then(Value::as_str)
    {
        state.response_id = Some(id.to_string());
    }
    if state.response_model.is_none()
        && let Some(model) = payload.get("model").and_then(Value::as_str)
    {
        state.response_model = Some(model.to_string());
    }

    if payload.get("usage").is_some() && payload.get("choices").is_none() {
        state.pending_usage = Some(parse_usage(payload.get("usage")));
        return Ok(out);
    }

    let choices = payload
        .get("choices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for choice in &choices {
        let delta = choice.get("delta").unwrap_or(&Value::Null);

        if let Some(content_delta) = delta.get("content").and_then(Value::as_str)
            && !content_delta.is_empty()
        {
            state.accumulated_text.push_str(content_delta);
            let text_id = "text_0".to_string();
            if !state.emitted_text_start {
                state.emitted_text_start = true;
                out.push(StreamEvent {
                    event_type: StreamEventType::TextStart,
                    delta: None,
                    text_id: Some(text_id.clone()),
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

            out.push(StreamEvent {
                event_type: StreamEventType::TextDelta,
                delta: Some(content_delta.to_string()),
                text_id: Some(text_id),
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

        if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for tool_call in tool_calls {
                let index = tool_call.get("index").and_then(Value::as_u64).unwrap_or(0);

                let id = tool_call
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| state.tool_calls.get(&index).map(|tc| tc.id.clone()))
                    .unwrap_or_else(|| format!("call_{index}"));

                let name = tool_call
                    .pointer("/function/name")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| state.tool_calls.get(&index).map(|tc| tc.name.clone()))
                    .unwrap_or_else(|| "unknown_tool".to_string());

                let arguments_delta = tool_call
                    .pointer("/function/arguments")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                let call_state = state
                    .tool_calls
                    .entry(index)
                    .or_insert_with(|| ToolCallState {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: String::new(),
                        started: false,
                    });

                call_state.id.clone_from(&id);
                call_state.name.clone_from(&name);
                call_state.arguments.push_str(&arguments_delta);

                if !call_state.started {
                    call_state.started = true;
                    out.push(tool_call_event(
                        StreamEventType::ToolCallStart,
                        ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: Value::String(String::new()),
                            raw_arguments: Some(String::new()),
                            parse_error: None,
                        },
                        payload.clone(),
                    ));
                }

                if !arguments_delta.is_empty() {
                    out.push(tool_call_event(
                        StreamEventType::ToolCallDelta,
                        ToolCall {
                            id,
                            name,
                            arguments: Value::String(arguments_delta.clone()),
                            raw_arguments: Some(arguments_delta),
                            parse_error: None,
                        },
                        payload.clone(),
                    ));
                }
            }
        }

        if let Some(finish_reason) = choice.get("finish_reason").and_then(Value::as_str) {
            let reason = map_finish_reason(Some(finish_reason));

            if state.emitted_text_start && !state.seen_finish_text_ids.contains("text_0") {
                state.seen_finish_text_ids.insert("text_0".to_string());
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

            if reason == crate::types::finish_reason::Reason::ToolCalls {
                for tool_state in state.tool_calls.values() {
                    let (parsed_arguments, parse_error) =
                        parse_tool_arguments(&tool_state.arguments);
                    out.push(tool_call_event(
                        StreamEventType::ToolCallEnd,
                        ToolCall {
                            id: tool_state.id.clone(),
                            name: tool_state.name.clone(),
                            arguments: parsed_arguments,
                            raw_arguments: Some(tool_state.arguments.clone()),
                            parse_error,
                        },
                        payload.clone(),
                    ));
                }
            }

            if !state.finished {
                state.finished = true;
                let usage = choice
                    .get("usage")
                    .map(|value| parse_usage(Some(value)))
                    .or_else(|| payload.get("usage").map(|value| parse_usage(Some(value))))
                    .or_else(|| state.pending_usage.clone())
                    .unwrap_or_default();

                let fr = FinishReason {
                    reason,
                    raw: Some(finish_reason.to_string()),
                };

                // Build accumulated content for the full response
                let mut content = Vec::new();
                let text = std::mem::take(&mut state.accumulated_text);
                if !text.is_empty() {
                    content.push(ContentPart::Text { text });
                }
                for tool_state in state.tool_calls.values() {
                    let (parsed_args, _) = parse_tool_arguments(&tool_state.arguments);
                    content.push(ContentPart::ToolCall {
                        tool_call: ToolCallData {
                            id: tool_state.id.clone(),
                            name: tool_state.name.clone(),
                            arguments: parsed_args,
                            call_type: "function".to_string(),
                        },
                    });
                }

                let response = Response {
                    id: state
                        .response_id
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    model: state
                        .response_model
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    provider: "openai_chat_completions".to_string(),
                    message: Message {
                        role: Role::Assistant,
                        content,
                        name: None,
                        tool_call_id: None,
                    },
                    finish_reason: fr.clone(),
                    usage: usage.clone(),
                    raw: Some(payload.clone()),
                    warnings: None,
                    rate_limit: state.rate_limit.clone(),
                };

                out.push(StreamEvent {
                    event_type: StreamEventType::Finish,
                    delta: None,
                    text_id: None,
                    reasoning_delta: None,
                    tool_call: None,
                    finish_reason: Some(fr),
                    usage: Some(usage),
                    response: Some(Box::new(response)),
                    error: None,
                    warnings: None,
                    raw: Some(payload.clone()),
                });
            }
        }
    }

    if choices.is_empty() {
        // Check for error payloads before falling through to provider event
        if let Some(error) = payload.get("error") {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Chat Completions stream error")
                .to_string();
            let err = translate_error(SdkError::Server {
                message,
                details: ProviderDetails {
                    provider: Some("openai_chat_completions".to_string()),
                    raw: Some(payload.clone()),
                    retryable: true,
                    ..ProviderDetails::default()
                },
            });
            state.finished = true;
            out.push(StreamEvent::error(err));
        } else {
            out.push(provider_event(payload));
        }
    }

    Ok(out)
}

impl crate::providers::common::stream::SseStreamState for OpenAIChatCompletionsStreamState {
    fn translate_event(&mut self, event: &SseEvent) -> SdkResult<Vec<StreamEvent>> {
        translate_sse_event(event, self)
    }

    fn on_stream_end(&mut self) -> Vec<StreamEvent> {
        if self.emitted_stream_start {
            return Vec::new();
        }
        self.emitted_stream_start = true;
        vec![StreamEvent::stream_start()]
    }
}

/// Translate a full parsed SSE stream into unified stream events.
#[must_use]
pub fn translate_sse_stream<'a>(
    sse_stream: std::pin::Pin<Box<dyn futures::Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
    rate_limit: Option<RateLimitInfo>,
) -> crate::provider::BoxStream<'a, SdkResult<StreamEvent>> {
    crate::providers::common::stream::translate_sse_stream(
        sse_stream,
        OpenAIChatCompletionsStreamState::with_rate_limit(rate_limit),
    )
}

fn provider_event(payload: Value) -> StreamEvent {
    StreamEvent::provider_event(payload)
}

fn tool_call_event(event_type: StreamEventType, tool_call: ToolCall, raw: Value) -> StreamEvent {
    StreamEvent::tool_call_event(event_type, tool_call, raw)
}

fn parse_tool_arguments(raw: &str) -> (Value, Option<String>) {
    ToolCall::parse_arguments(raw)
}
