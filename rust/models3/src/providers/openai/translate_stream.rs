use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

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

/// Stateful translator for OpenAI Responses API SSE events.
#[derive(Debug, Clone, Default)]
pub struct OpenAIStreamState {
    emitted_stream_start: bool,
    started_text_ids: HashSet<String>,
    tool_calls: HashMap<String, ToolCallState>,
    rate_limit: Option<RateLimitInfo>,
    /// Whether we have emitted a ReasoningStart event for the current reasoning block.
    emitted_reasoning_start: bool,
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
/// mapped OpenAI event requires JSON decoding.
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
        // Pre-populate tool call state when the output item is first announced.
        // The function name is only available here; subsequent argument-delta
        // events do NOT carry it.
        "response.output_item.added" => {
            // Upstream references:
            // https://github.com/openai/codex/blob/main/codex-rs/codex-api/src/sse/responses.rs
            // https://github.com/openai/codex/blob/main/codex-rs/protocol/src/models.rs
            // The stream can announce non-function tool calls as output items.
            let item = payload.get("item").unwrap_or(&payload);
            let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
            match item_type {
                "function_call" => {
                    let call_id = item
                        .get("call_id")
                        .and_then(Value::as_str)
                        .or_else(|| item.get("id").and_then(Value::as_str))
                        .unwrap_or("call_0")
                        .to_string();
                    let name = item
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown_tool")
                        .to_string();

                    state.tool_calls.insert(
                        call_id.clone(),
                        ToolCallState {
                            name: name.clone(),
                            arguments: String::new(),
                        },
                    );

                    out.push(tool_call_event(
                        StreamEventType::ToolCallStart,
                        ToolCall {
                            id: call_id,
                            name,
                            arguments: Value::String(String::new()),
                            raw_arguments: Some(String::new()),
                            parse_error: None,
                        },
                        payload,
                    ));
                }
                "custom_tool_call" | "local_shell_call" => {
                    let (call_id, name, _arguments, raw_arguments, _parse_error) =
                        normalize_non_function_tool_call(item);

                    state.tool_calls.insert(
                        call_id.clone(),
                        ToolCallState {
                            name: name.clone(),
                            arguments: raw_arguments,
                        },
                    );

                    out.push(tool_call_event(
                        StreamEventType::ToolCallStart,
                        ToolCall {
                            id: call_id.clone(),
                            name: name.clone(),
                            arguments: Value::Object(Map::new()),
                            raw_arguments: Some(String::new()),
                            parse_error: None,
                        },
                        payload,
                    ));
                }
                _ => out.push(provider_event(payload)),
            }
        }
        // Reasoning summary part added — emit ReasoningStart.
        // OpenAI Responses API sends this when a reasoning summary block begins.
        "response.reasoning_summary_part.added" => {
            if state.emitted_reasoning_start {
                out.push(provider_event(payload));
            } else {
                state.emitted_reasoning_start = true;
                out.push(StreamEvent {
                    event_type: StreamEventType::ReasoningStart,
                    delta: None,
                    text_id: Some("reasoning_0".to_string()),
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
        // Reasoning summary text delta — emit ReasoningDelta with the text.
        "response.reasoning_summary_text.delta" => {
            if !state.emitted_reasoning_start {
                state.emitted_reasoning_start = true;
                out.push(StreamEvent {
                    event_type: StreamEventType::ReasoningStart,
                    delta: None,
                    text_id: Some("reasoning_0".to_string()),
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

            let delta = payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            if !delta.is_empty() {
                out.push(StreamEvent {
                    event_type: StreamEventType::ReasoningDelta,
                    delta: None,
                    text_id: Some("reasoning_0".to_string()),
                    reasoning_delta: Some(delta),
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
        // Reasoning summary text done — emit ReasoningEnd.
        "response.reasoning_summary_text.done" | "response.reasoning_summary_part.done" => {
            if state.emitted_reasoning_start {
                state.emitted_reasoning_start = false;
                out.push(StreamEvent {
                    event_type: StreamEventType::ReasoningEnd,
                    delta: None,
                    text_id: Some("reasoning_0".to_string()),
                    reasoning_delta: None,
                    tool_call: None,
                    finish_reason: None,
                    usage: None,
                    response: None,
                    error: None,
                    warnings: None,
                    raw: Some(payload),
                });
            } else {
                out.push(provider_event(payload));
            }
        }
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
                .map(ToString::to_string)
                .or_else(|| state.tool_calls.get(&call_id).map(|tc| tc.name.clone()))
                .unwrap_or_else(|| "unknown_tool".to_string());
            let delta = payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            let already_started = state.tool_calls.contains_key(&call_id);
            let call_state =
                state
                    .tool_calls
                    .entry(call_id.clone())
                    .or_insert_with(|| ToolCallState {
                        name: name.clone(),
                        arguments: String::new(),
                    });

            // Only emit ToolCallStart if not already started by
            // response.output_item.added (or a prior delta).
            if !already_started {
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

            // Only update the stored name when the payload carries a real one;
            // avoid overwriting a good name from output_item.added with a
            // fallback "unknown_tool".
            if name != "unknown_tool" {
                call_state.name.clone_from(&name);
            }
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
                    // Prefer call_id to stay consistent with output_item.added
                    // and extract_call_id (used by argument-delta events).
                    let call_id = item
                        .get("call_id")
                        .and_then(Value::as_str)
                        .or_else(|| item.get("id").and_then(Value::as_str))
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
                "custom_tool_call" | "local_shell_call" => {
                    let (call_id, name, arguments, raw_arguments, parse_error) =
                        normalize_non_function_tool_call(item);

                    // Non-function calls usually arrive only at `.done`.
                    // If we didn't see an earlier start event, synthesize one.
                    if !state.tool_calls.contains_key(&call_id) {
                        out.push(tool_call_event(
                            StreamEventType::ToolCallStart,
                            ToolCall {
                                id: call_id.clone(),
                                name: name.clone(),
                                arguments: Value::Object(Map::new()),
                                raw_arguments: Some(String::new()),
                                parse_error: None,
                            },
                            payload.clone(),
                        ));
                    }

                    out.push(tool_call_event(
                        StreamEventType::ToolCallEnd,
                        ToolCall {
                            id: call_id.clone(),
                            name,
                            arguments,
                            raw_arguments: Some(raw_arguments),
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

fn normalize_non_function_tool_call(
    item: &Value,
) -> (String, String, Value, String, Option<String>) {
    // Upstream fixture/examples:
    // https://github.com/openai/codex/blob/main/codex-rs/core/tests/common/responses.rs
    // Normalizes Responses API `custom_tool_call` / `local_shell_call`
    // into the unified tool-call shape expected by stencila-agents.
    let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
    let call_id = item
        .get("call_id")
        .and_then(Value::as_str)
        .or_else(|| item.get("id").and_then(Value::as_str))
        .unwrap_or("call_0")
        .to_string();

    match item_type {
        "custom_tool_call" => {
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown_tool")
                .to_string();
            let input = item
                .get("input")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            let (parsed, parse_error) = parse_tool_arguments(&input);
            let arguments = if parse_error.is_none() && parsed.is_object() {
                parsed
            } else if name == "apply_patch" {
                json!({ "patch": input })
            } else {
                json!({ "input": input })
            };
            (call_id, name, arguments, input, parse_error)
        }
        "local_shell_call" => {
            let action = item
                .get("action")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            let command = action
                .get("command")
                .and_then(Value::as_array)
                .map(|parts| {
                    parts
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            let arguments = if command.is_empty() {
                Value::Object(Map::new())
            } else {
                json!({ "command": command })
            };
            let raw_arguments = serde_json::to_string(&action).unwrap_or_default();
            (call_id, "shell".to_string(), arguments, raw_arguments, None)
        }
        _ => (
            call_id,
            "unknown_tool".to_string(),
            Value::Object(Map::new()),
            String::new(),
            None,
        ),
    }
}
