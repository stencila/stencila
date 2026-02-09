use std::collections::HashMap;

use serde_json::Value;

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::sse::SseEvent;
use crate::providers::anthropic::translate_error::translate_error;
use crate::providers::anthropic::translate_response::{map_finish_reason, parse_usage};
use crate::types::content::{ContentPart, ThinkingData, ToolCallData};
use crate::types::finish_reason::FinishReason;
use crate::types::message::Message;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::ToolCall;

/// Accumulated state for a tool call being streamed.
#[derive(Debug, Clone, Default)]
struct ToolCallState {
    id: String,
    name: String,
    arguments: String,
}

/// Stateful translator for Anthropic Messages API SSE events.
#[derive(Debug, Clone, Default)]
pub struct AnthropicStreamState {
    emitted_stream_start: bool,
    /// Maps block index to block type ("text", "`tool_use`", "thinking")
    block_types: HashMap<u64, String>,
    /// Maps block index to accumulated tool call state
    tool_calls: HashMap<u64, ToolCallState>,
    /// Input usage from `message_start` event
    input_usage: Option<Value>,
    /// Output usage from `message_delta` event
    output_usage: Option<Value>,
    /// Finish reason from `message_delta` event
    finish_reason: Option<String>,
    /// Message ID from `message_start` event.
    message_id: String,
    /// Model from `message_start` event.
    model: String,
    /// Rate limit info from response headers.
    rate_limit: Option<RateLimitInfo>,
    /// Accumulated content parts for building the full response on FINISH.
    accumulated_content: Vec<ContentPart>,
    /// Accumulated text per block index for text content.
    accumulated_text: HashMap<u64, String>,
    /// Accumulated thinking text per block index.
    accumulated_thinking: HashMap<u64, String>,
    /// Accumulated thinking signatures per block index (delivered in `content_block_stop`).
    thinking_signatures: HashMap<u64, String>,
}

impl AnthropicStreamState {
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
/// Anthropic SSE events use the `event:` field (the `event_type` on `SseEvent`)
/// to distinguish event kinds, not a JSON `type` field inside the data.
///
/// # Errors
///
/// Returns `SdkError::Stream` if the SSE payload is not valid JSON.
pub fn translate_sse_event(
    event: &SseEvent,
    state: &mut AnthropicStreamState,
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
        message: format!("invalid Anthropic SSE payload JSON: {e}"),
    })?;

    match event.event_type.as_str() {
        "message_start" => handle_message_start(&payload, state),
        "content_block_start" => handle_block_start(payload, state, &mut out),
        "content_block_delta" => handle_block_delta(payload, state, &mut out),
        "content_block_stop" => handle_block_stop(payload, state, &mut out),
        "message_delta" => handle_message_delta(&payload, state),
        "message_stop" => handle_message_stop(payload, state, &mut out),
        "error" => handle_error(payload, &mut out),
        "ping" => {}
        _ => out.push(provider_event(payload)),
    }

    Ok(out)
}

fn handle_message_start(payload: &Value, state: &mut AnthropicStreamState) {
    if let Some(message) = payload.get("message") {
        state.message_id = message
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        state.model = message
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        state.input_usage = message.get("usage").cloned();
    }
}

fn handle_block_start(
    payload: Value,
    state: &mut AnthropicStreamState,
    out: &mut Vec<StreamEvent>,
) {
    let index = payload.get("index").and_then(Value::as_u64).unwrap_or(0);
    let content_block = payload.get("content_block").unwrap_or(&payload);
    let block_type = content_block
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    state.block_types.insert(index, block_type.to_string());

    match block_type {
        "text" => out.push(simple_event(
            StreamEventType::TextStart,
            Some(format!("block_{index}")),
            payload,
        )),
        "tool_use" => {
            let id = content_block
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let name = content_block
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            state.tool_calls.insert(
                index,
                ToolCallState {
                    id: id.clone(),
                    name: name.clone(),
                    arguments: String::new(),
                },
            );

            out.push(tool_call_event(
                StreamEventType::ToolCallStart,
                ToolCall {
                    id,
                    name,
                    arguments: Value::Object(serde_json::Map::new()),
                    raw_arguments: Some(String::new()),
                    parse_error: None,
                },
                payload,
            ));
        }
        "thinking" => out.push(simple_event(
            StreamEventType::ReasoningStart,
            Some(format!("block_{index}")),
            payload,
        )),
        "redacted_thinking" => {
            // Redacted thinking blocks carry their data in `content_block_start`
            // and have no deltas. Extract the data now; `content_block_stop` will
            // finalize the block.
            let data = content_block
                .get("data")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            // Store in accumulated_thinking so we can build the ContentPart on stop
            state.accumulated_thinking.insert(index, data);
            out.push(simple_event(
                StreamEventType::ReasoningStart,
                Some(format!("block_{index}")),
                payload,
            ));
        }
        _ => out.push(provider_event(payload)),
    }
}

fn handle_block_delta(
    payload: Value,
    state: &mut AnthropicStreamState,
    out: &mut Vec<StreamEvent>,
) {
    let index = payload.get("index").and_then(Value::as_u64).unwrap_or(0);
    let delta = payload.get("delta").unwrap_or(&Value::Null);
    let delta_type = delta
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match delta_type {
        "text_delta" => {
            let text = delta
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            state
                .accumulated_text
                .entry(index)
                .or_default()
                .push_str(&text);
            out.push(StreamEvent {
                event_type: StreamEventType::TextDelta,
                delta: Some(text),
                text_id: Some(format!("block_{index}")),
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
        "input_json_delta" => {
            let partial_json = delta
                .get("partial_json")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            if let Some(tc_state) = state.tool_calls.get_mut(&index) {
                tc_state.arguments.push_str(&partial_json);
                out.push(tool_call_event(
                    StreamEventType::ToolCallDelta,
                    ToolCall {
                        id: tc_state.id.clone(),
                        name: tc_state.name.clone(),
                        arguments: Value::String(partial_json.clone()),
                        raw_arguments: Some(partial_json),
                        parse_error: None,
                    },
                    payload,
                ));
            }
        }
        "thinking_delta" => {
            let thinking_text = delta
                .get("thinking")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            state
                .accumulated_thinking
                .entry(index)
                .or_default()
                .push_str(&thinking_text);
            out.push(StreamEvent {
                event_type: StreamEventType::ReasoningDelta,
                delta: None,
                text_id: Some(format!("block_{index}")),
                reasoning_delta: Some(thinking_text),
                tool_call: None,
                finish_reason: None,
                usage: None,
                response: None,
                error: None,
                warnings: None,
                raw: Some(payload),
            });
        }
        "signature_delta" => {
            let signature = delta
                .get("signature")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            state
                .thinking_signatures
                .entry(index)
                .or_default()
                .push_str(&signature);
        }
        _ => out.push(provider_event(payload)),
    }
}

fn handle_block_stop(payload: Value, state: &mut AnthropicStreamState, out: &mut Vec<StreamEvent>) {
    let index = payload.get("index").and_then(Value::as_u64).unwrap_or(0);

    let Some(block_type) = state.block_types.remove(&index) else {
        return;
    };

    match block_type.as_str() {
        "text" => {
            if let Some(text) = state.accumulated_text.remove(&index) {
                state.accumulated_content.push(ContentPart::Text { text });
            }
            out.push(simple_event(
                StreamEventType::TextEnd,
                Some(format!("block_{index}")),
                payload,
            ));
        }
        "tool_use" => {
            if let Some(tc_state) = state.tool_calls.remove(&index) {
                let (parsed_arguments, parse_error) = parse_tool_arguments(&tc_state.arguments);
                // Accumulate as content part for the response
                state.accumulated_content.push(ContentPart::ToolCall {
                    tool_call: ToolCallData {
                        id: tc_state.id.clone(),
                        name: tc_state.name.clone(),
                        arguments: parsed_arguments.clone(),
                        call_type: "function".to_string(),
                    },
                });
                out.push(tool_call_event(
                    StreamEventType::ToolCallEnd,
                    ToolCall {
                        id: tc_state.id,
                        name: tc_state.name,
                        arguments: parsed_arguments,
                        raw_arguments: Some(tc_state.arguments),
                        parse_error,
                    },
                    payload,
                ));
            }
        }
        "thinking" => {
            let signature = state.thinking_signatures.remove(&index);
            if let Some(text) = state.accumulated_thinking.remove(&index) {
                state.accumulated_content.push(ContentPart::Thinking {
                    thinking: ThinkingData {
                        text,
                        signature,
                        redacted: false,
                    },
                });
            }
            out.push(simple_event(
                StreamEventType::ReasoningEnd,
                Some(format!("block_{index}")),
                payload,
            ));
        }
        "redacted_thinking" => {
            if let Some(data) = state.accumulated_thinking.remove(&index) {
                state
                    .accumulated_content
                    .push(ContentPart::RedactedThinking {
                        thinking: ThinkingData {
                            text: data,
                            signature: None,
                            redacted: true,
                        },
                    });
            }
            out.push(simple_event(
                StreamEventType::ReasoningEnd,
                Some(format!("block_{index}")),
                payload,
            ));
        }
        _ => out.push(provider_event(payload)),
    }
}

fn handle_message_delta(payload: &Value, state: &mut AnthropicStreamState) {
    if let Some(delta) = payload.get("delta") {
        state.finish_reason = delta
            .get("stop_reason")
            .and_then(Value::as_str)
            .map(ToString::to_string);
    }
    state.output_usage = payload.get("usage").cloned();
}

fn handle_message_stop(
    payload: Value,
    state: &mut AnthropicStreamState,
    out: &mut Vec<StreamEvent>,
) {
    let mut usage = parse_usage(state.input_usage.as_ref());
    let output_usage = parse_usage(state.output_usage.as_ref());
    usage = usage + output_usage;

    let content = std::mem::take(&mut state.accumulated_content);

    let reason = if content
        .iter()
        .any(|part| matches!(part, ContentPart::ToolCall { .. }))
    {
        crate::types::finish_reason::Reason::ToolCalls
    } else {
        map_finish_reason(state.finish_reason.as_deref())
    };

    let finish_reason = FinishReason {
        reason,
        raw: state.finish_reason.clone(),
    };

    let response = Response {
        id: state.message_id.clone(),
        model: state.model.clone(),
        provider: "anthropic".to_string(),
        message: Message {
            role: Role::Assistant,
            content,
            name: None,
            tool_call_id: None,
        },
        finish_reason: finish_reason.clone(),
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
        finish_reason: Some(finish_reason),
        usage: Some(usage),
        response: Some(Box::new(response)),
        error: None,
        warnings: None,
        raw: Some(payload),
    });
}

fn handle_error(payload: Value, out: &mut Vec<StreamEvent>) {
    let message = payload
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| payload.get("message").and_then(Value::as_str))
        .unwrap_or("anthropic stream failed")
        .to_string();

    let err = translate_error(SdkError::Server {
        message,
        details: ProviderDetails {
            provider: Some("anthropic".to_string()),
            raw: Some(payload),
            retryable: true,
            ..ProviderDetails::default()
        },
    });
    out.push(StreamEvent::error(err));
}

impl crate::providers::common::stream::SseStreamState for AnthropicStreamState {
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
        AnthropicStreamState::with_rate_limit(rate_limit),
    )
}

fn simple_event(event_type: StreamEventType, text_id: Option<String>, raw: Value) -> StreamEvent {
    StreamEvent::simple(event_type, text_id, raw)
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
