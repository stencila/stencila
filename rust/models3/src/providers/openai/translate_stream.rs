use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::sse::SseEvent;
use crate::providers::openai::translate_error::translate_error;
use crate::providers::openai::translate_response::translate_response;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::ToolCall;

#[derive(Debug, Clone, Default)]
struct ToolCallState {
    name: String,
    arguments: String,
}

/// Stateful translator for `OpenAI` Responses API SSE events.
#[derive(Debug, Clone, Default)]
pub struct OpenAIStreamState {
    emitted_stream_start: bool,
    started_text_ids: HashSet<String>,
    tool_calls: HashMap<String, ToolCallState>,
    rate_limit: Option<RateLimitInfo>,
}

impl OpenAIStreamState {
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
/// Returns `SdkError::Stream` if the SSE payload is not valid JSON when a
/// mapped `OpenAI` event requires JSON decoding.
#[allow(clippy::too_many_lines)]
pub fn translate_sse_event(
    event: &SseEvent,
    state: &mut OpenAIStreamState,
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
        message: format!("invalid OpenAI SSE payload JSON: {e}"),
    })?;

    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or(event.event_type.as_str());

    match event_type {
        "response.output_text.delta" => {
            let delta = payload
                .get("delta")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::Stream {
                    message: "response.output_text.delta missing `delta` field".to_string(),
                })?
                .to_string();

            let text_id = extract_text_id(&payload);
            if state.started_text_ids.insert(text_id.clone()) {
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
                delta: Some(delta),
                text_id: Some(text_id),
                reasoning_delta: None,
                tool_call: None,
                finish_reason: None,
                usage: None,
                response: None,
                error: None,
                warnings: None,
                raw: Some(payload),
            });
        }
        "response.function_call_arguments.delta" => {
            let call_id = extract_call_id(&payload);
            let name = payload
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| payload.pointer("/item/name").and_then(Value::as_str))
                .unwrap_or("unknown_tool")
                .to_string();
            let delta = payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            let call_state =
                state
                    .tool_calls
                    .entry(call_id.clone())
                    .or_insert_with(|| ToolCallState {
                        name: name.clone(),
                        arguments: String::new(),
                    });

            if call_state.arguments.is_empty() {
                out.push(tool_call_event(
                    StreamEventType::ToolCallStart,
                    ToolCall {
                        id: call_id.clone(),
                        name: name.clone(),
                        arguments: Value::String(String::new()),
                        raw_arguments: Some(String::new()),
                        parse_error: None,
                    },
                    payload.clone(),
                ));
            }

            call_state.name = name;
            call_state.arguments.push_str(&delta);

            out.push(tool_call_event(
                StreamEventType::ToolCallDelta,
                ToolCall {
                    id: call_id,
                    name: call_state.name.clone(),
                    arguments: Value::String(delta.clone()),
                    raw_arguments: Some(delta),
                    parse_error: None,
                },
                payload,
            ));
        }
        "response.output_item.done" => {
            let item = payload.get("item").unwrap_or(&payload);
            let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
            match item_type {
                "message" | "output_text" => {
                    let text_id = extract_text_id(item);
                    if state.started_text_ids.contains(&text_id) {
                        out.push(StreamEvent {
                            event_type: StreamEventType::TextEnd,
                            delta: None,
                            text_id: Some(text_id),
                            reasoning_delta: None,
                            tool_call: None,
                            finish_reason: None,
                            usage: None,
                            response: None,
                            error: None,
                            warnings: None,
                            raw: Some(payload),
                        });
                    }
                }
                "function_call" => {
                    let call_id = item
                        .get("id")
                        .and_then(Value::as_str)
                        .or_else(|| item.get("call_id").and_then(Value::as_str))
                        .unwrap_or("call_0")
                        .to_string();

                    let mut arguments = item
                        .get("arguments")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .unwrap_or_default();

                    let name = item
                        .get("name")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .or_else(|| {
                            state
                                .tool_calls
                                .get(&call_id)
                                .map(|entry| entry.name.clone())
                        })
                        .unwrap_or_else(|| "unknown_tool".to_string());

                    if arguments.is_empty()
                        && let Some(stateful) = state.tool_calls.get(&call_id)
                    {
                        arguments.clone_from(&stateful.arguments);
                    }

                    let (parsed_arguments, parse_error) = parse_tool_arguments(&arguments);
                    out.push(tool_call_event(
                        StreamEventType::ToolCallEnd,
                        ToolCall {
                            id: call_id.clone(),
                            name,
                            arguments: parsed_arguments,
                            raw_arguments: Some(arguments),
                            parse_error,
                        },
                        payload,
                    ));
                    state.tool_calls.remove(&call_id);
                }
                _ => {
                    out.push(provider_event(payload));
                }
            }
        }
        "response.completed" => {
            let response_payload = payload.get("response").cloned().unwrap_or(payload.clone());
            let mut response = translate_response(response_payload.clone(), None)?;
            if response.rate_limit.is_none() {
                response.rate_limit.clone_from(&state.rate_limit);
            }

            out.push(StreamEvent {
                event_type: StreamEventType::Finish,
                delta: None,
                text_id: None,
                reasoning_delta: None,
                tool_call: None,
                finish_reason: Some(response.finish_reason.clone()),
                usage: Some(response.usage.clone()),
                response: Some(Box::new(response)),
                error: None,
                warnings: None,
                raw: Some(payload),
            });
        }
        "response.failed" | "error" => {
            let message = payload
                .pointer("/error/message")
                .and_then(Value::as_str)
                .or_else(|| payload.get("message").and_then(Value::as_str))
                .unwrap_or("openai stream failed")
                .to_string();

            let err = translate_error(SdkError::Server {
                message,
                details: ProviderDetails {
                    provider: Some("openai".to_string()),
                    raw: Some(payload.clone()),
                    retryable: true,
                    ..ProviderDetails::default()
                },
            });
            out.push(StreamEvent::error(err));
        }
        // In-progress events are surfaced as provider passthrough events.
        _ => {
            out.push(provider_event(payload));
        }
    }

    Ok(out)
}

impl crate::providers::common::stream::SseStreamState for OpenAIStreamState {
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

/// Translate a full SSE stream into unified stream events.
#[must_use]
pub fn translate_sse_stream<'a>(
    sse_stream: std::pin::Pin<Box<dyn futures::Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
    rate_limit: Option<RateLimitInfo>,
) -> crate::provider::BoxStream<'a, SdkResult<StreamEvent>> {
    crate::providers::common::stream::translate_sse_stream(
        sse_stream,
        OpenAIStreamState::with_rate_limit(rate_limit),
    )
}

fn provider_event(payload: Value) -> StreamEvent {
    StreamEvent::provider_event(payload)
}

fn tool_call_event(event_type: StreamEventType, tool_call: ToolCall, raw: Value) -> StreamEvent {
    StreamEvent::tool_call_event(event_type, tool_call, raw)
}

fn extract_text_id(payload: &Value) -> String {
    payload
        .get("text_id")
        .and_then(Value::as_str)
        .or_else(|| payload.get("item_id").and_then(Value::as_str))
        .or_else(|| payload.pointer("/item/id").and_then(Value::as_str))
        .map_or_else(
            || {
                let output_index = payload
                    .get("output_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let content_index = payload
                    .get("content_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                format!("text_{output_index}_{content_index}")
            },
            ToString::to_string,
        )
}

fn extract_call_id(payload: &Value) -> String {
    payload
        .get("call_id")
        .and_then(Value::as_str)
        .or_else(|| payload.get("id").and_then(Value::as_str))
        .or_else(|| payload.get("item_id").and_then(Value::as_str))
        .map_or_else(|| "call_0".to_string(), ToString::to_string)
}

fn parse_tool_arguments(raw: &str) -> (Value, Option<String>) {
    ToolCall::parse_arguments(raw)
}
