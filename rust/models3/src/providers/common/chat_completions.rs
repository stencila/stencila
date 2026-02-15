//! Shared Chat Completions translation logic.
//!
//! Both the `openai_chat_completions` and `mistral` adapters use the same
//! wire format (`/v1/chat/completions`). This module extracts the shared
//! request, response, and stream translation code, parameterized by a
//! [`ChatCompletionsConfig`].

use std::collections::{BTreeMap, HashSet};

use reqwest::header::HeaderMap;
use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::sse::SseEvent;
use crate::providers::common::openai_shared::{
    image_to_openai_url, parse_custom_headers, translate_response_format, translate_tool_choice,
    value_to_json_string,
};
use crate::providers::common::translate_error::{ErrorConfig, translate_error as common_translate};
use crate::types::content::{ContentPart, ToolCallData};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::{ToolCall, ToolDefinition};
use crate::types::usage::Usage;

// ─── Configuration ───────────────────────────────────────────────────────

/// How to handle `null` content values for assistant messages with tool calls
/// but no text content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NullContentHandling {
    /// Send `"content": null` (OpenAI accepts this).
    ExplicitNull,
    /// Omit the `content` key entirely (Mistral rejects `null` values).
    OmitKey,
}

/// Configuration for a Chat Completions adapter.
#[derive(Debug, Clone)]
pub(crate) struct ChatCompletionsConfig {
    /// Provider name for error messages and response attribution.
    pub provider_name: &'static str,
    /// Namespaces to check for provider-specific options.
    pub option_namespaces: &'static [&'static str],
    /// Namespaces to check when rejecting built-in tools (Responses-only).
    pub builtin_tools_guard_namespaces: &'static [&'static str],
    /// How to handle null content in assistant messages.
    pub null_content_handling: NullContentHandling,
}

// ─── Request Translation ─────────────────────────────────────────────────

/// Chat Completions translated request body + per-request headers.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedChatCompletionsRequest {
    pub body: Value,
    pub headers: HeaderMap,
}

/// Translate a unified request into a Chat Completions request.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` for unsupported content or invalid
/// provider options.
pub(crate) fn translate_request(
    request: &Request,
    stream: bool,
    config: &ChatCompletionsConfig,
) -> SdkResult<TranslatedChatCompletionsRequest> {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(request.model.clone()));

    let mut messages = Vec::new();
    for message in &request.messages {
        translate_message(message, &mut messages, config)?;
    }
    body.insert("messages".to_string(), Value::Array(messages));

    if let Some(tools) = &request.tools {
        let translated_tools: SdkResult<Vec<Value>> = tools
            .iter()
            .map(|t| translate_tool_definition(t, config))
            .collect();
        let translated_tools = translated_tools?;
        if !translated_tools.is_empty() {
            body.insert("tools".to_string(), Value::Array(translated_tools));
        }
    }

    if let Some(tool_choice) = &request.tool_choice {
        body.insert(
            "tool_choice".to_string(),
            translate_tool_choice(tool_choice),
        );
    }

    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(top_p) = request.top_p {
        body.insert("top_p".to_string(), json!(top_p));
    }
    if let Some(max_tokens) = request.max_tokens {
        body.insert("max_tokens".to_string(), json!(max_tokens));
    }
    if let Some(stop_sequences) = &request.stop_sequences {
        body.insert("stop".to_string(), json!(stop_sequences));
    }
    if let Some(metadata) = &request.metadata {
        body.insert("metadata".to_string(), json!(metadata));
    }

    if let Some(response_format) = &request.response_format {
        body.insert(
            "response_format".to_string(),
            translate_response_format(response_format),
        );
    }

    if stream {
        body.insert("stream".to_string(), Value::Bool(true));
    }

    let mut headers = HeaderMap::new();
    // Check each option namespace in order, use the first match
    let options = config
        .option_namespaces
        .iter()
        .find_map(|ns| request.provider_options_for(ns));
    if let Some(options) = options {
        apply_provider_options(options, &mut body, &mut headers, config)?;
    }

    // Guardrail: this adapter intentionally does not support Responses-only features.
    let builtin_tools_keys = ["built_in_tools", "builtin_tools"];
    for ns in config.builtin_tools_guard_namespaces {
        if let Some(options) = request.provider_options_for(ns)
            && builtin_tools_keys.iter().any(|k| options.get(*k).is_some())
        {
            return Err(SdkError::InvalidRequest {
                message: format!(
                    "{} adapter does not support built-in tools (Responses-only)",
                    config.provider_name
                ),
                details: ProviderDetails {
                    provider: Some(config.provider_name.to_string()),
                    ..ProviderDetails::default()
                },
            });
        }
    }

    Ok(TranslatedChatCompletionsRequest {
        body: Value::Object(body),
        headers,
    })
}

#[allow(clippy::too_many_lines)]
fn translate_message(
    message: &crate::types::message::Message,
    messages: &mut Vec<Value>,
    config: &ChatCompletionsConfig,
) -> SdkResult<()> {
    match message.role {
        Role::System | Role::Developer => {
            let text = message
                .content
                .iter()
                .map(|part| match part {
                    ContentPart::Text { text } => Ok(text.clone()),
                    _ => Err(SdkError::InvalidRequest {
                        message: "system/developer messages must be text-only in Chat Completions"
                            .to_string(),
                        details: ProviderDetails {
                            provider: Some(config.provider_name.to_string()),
                            ..ProviderDetails::default()
                        },
                    }),
                })
                .collect::<SdkResult<Vec<String>>>()?
                .join("\n\n");

            messages.push(json!({"role": "system", "content": text}));
        }
        Role::User => {
            let content = translate_user_content_parts(&message.content, config)?;
            messages.push(json!({"role": "user", "content": content}));
        }
        Role::Assistant => {
            let mut text_parts = Vec::new();
            let mut tool_calls = Vec::new();

            for part in &message.content {
                match part {
                    ContentPart::Text { text } => text_parts.push(text.clone()),
                    ContentPart::Thinking { thinking } => {
                        text_parts.push(thinking.text.clone());
                    }
                    ContentPart::RedactedThinking { .. } => {
                        // Opaque redacted thinking is dropped when translating
                        // across providers.
                    }
                    ContentPart::ToolCall { tool_call } => {
                        tool_calls.push(json!({
                            "id": tool_call.id,
                            "type": "function",
                            "function": {
                                "name": tool_call.name,
                                "arguments": value_to_json_string(&tool_call.arguments, config.provider_name)?
                            }
                        }));
                    }
                    ContentPart::Audio { .. }
                    | ContentPart::Document { .. }
                    | ContentPart::Image { .. }
                    | ContentPart::ToolResult { .. }
                    | ContentPart::Extension(_) => {
                        return Err(SdkError::InvalidRequest {
                            message: format!(
                                "unsupported assistant content in Chat Completions translation: {part:?}"
                            ),
                            details: ProviderDetails {
                                provider: Some(config.provider_name.to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }

            let mut assistant_message = Map::new();
            assistant_message.insert("role".to_string(), Value::String("assistant".to_string()));

            if text_parts.is_empty() {
                match config.null_content_handling {
                    NullContentHandling::ExplicitNull => {
                        assistant_message.insert("content".to_string(), Value::Null);
                    }
                    NullContentHandling::OmitKey => {
                        // Do not insert the content key at all.
                    }
                }
            } else {
                assistant_message.insert("content".to_string(), Value::String(text_parts.join("")));
            }

            if !tool_calls.is_empty() {
                assistant_message.insert("tool_calls".to_string(), Value::Array(tool_calls));
            }

            messages.push(Value::Object(assistant_message));
        }
        Role::Tool => {
            for part in &message.content {
                match part {
                    ContentPart::ToolResult { tool_result } => {
                        // Chat Completions API types tool content as `string`.
                        // Stringify non-string values (Mistral rejects objects).
                        let content_str = match &tool_result.content {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_result.tool_call_id,
                            "content": content_str
                        }));
                    }
                    ContentPart::Text { text } => {
                        let tool_call_id = message.tool_call_id.clone().ok_or_else(|| {
                            SdkError::InvalidRequest {
                                message: "tool-role text messages require tool_call_id".to_string(),
                                details: ProviderDetails {
                                    provider: Some(config.provider_name.to_string()),
                                    ..ProviderDetails::default()
                                },
                            }
                        })?;
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_call_id,
                            "content": text
                        }));
                    }
                    _ => {
                        return Err(SdkError::InvalidRequest {
                            message: format!(
                                "tool-role messages only support tool results/text for Chat Completions: {part:?}"
                            ),
                            details: ProviderDetails {
                                provider: Some(config.provider_name.to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn translate_user_content_parts(
    parts: &[ContentPart],
    config: &ChatCompletionsConfig,
) -> SdkResult<Value> {
    let mut items = Vec::new();
    for part in parts {
        match part {
            ContentPart::Text { text } => {
                items.push(json!({"type": "text", "text": text}));
            }
            ContentPart::Thinking { thinking } => {
                items.push(json!({"type": "text", "text": thinking.text}));
            }
            ContentPart::RedactedThinking { .. } => {
                // Opaque redacted thinking is dropped when switching providers.
            }
            ContentPart::Image { image } => {
                image.validate()?;
                let mut image_url_obj = serde_json::Map::new();
                image_url_obj.insert(
                    "url".to_string(),
                    Value::String(image_to_openai_url(image, config.provider_name)?),
                );
                if let Some(detail) = &image.detail {
                    image_url_obj.insert("detail".to_string(), Value::String(detail.clone()));
                }
                items.push(json!({
                    "type": "image_url",
                    "image_url": image_url_obj
                }));
            }
            _ => {
                return Err(SdkError::InvalidRequest {
                    message: format!(
                        "unsupported user content in Chat Completions translation: {part:?}"
                    ),
                    details: ProviderDetails {
                        provider: Some(config.provider_name.to_string()),
                        ..ProviderDetails::default()
                    },
                });
            }
        }
    }

    if items.len() == 1
        && let Some(text) = items[0].get("text").and_then(Value::as_str)
    {
        return Ok(Value::String(text.to_string()));
    }

    Ok(Value::Array(items))
}

fn translate_tool_definition(
    tool: &ToolDefinition,
    config: &ChatCompletionsConfig,
) -> SdkResult<Value> {
    let _ = config; // reserved for future per-provider differences
    tool.validate()?;
    Ok(json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters,
            "strict": tool.strict
        }
    }))
}

fn apply_provider_options(
    options: &Value,
    body: &mut Map<String, Value>,
    headers: &mut HeaderMap,
    config: &ChatCompletionsConfig,
) -> SdkResult<()> {
    let Some(options_obj) = options.as_object() else {
        return Err(SdkError::InvalidRequest {
            message: format!(
                "provider_options.{} must be an object",
                config.provider_name
            ),
            details: ProviderDetails {
                provider: Some(config.provider_name.to_string()),
                ..ProviderDetails::default()
            },
        });
    };

    parse_custom_headers(options_obj, headers, config.provider_name)?;

    for (key, value) in options_obj {
        if matches!(key.as_str(), "custom_headers" | "headers") {
            continue;
        }
        body.insert(key.clone(), value.clone());
    }

    Ok(())
}

// ─── Response Translation ────────────────────────────────────────────────

/// Translate a Chat Completions response into a unified response.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` if required fields are missing.
pub(crate) fn translate_response(
    raw_response: Value,
    headers: Option<&HeaderMap>,
    provider_name: &str,
) -> SdkResult<Response> {
    let id = raw_response
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "chat completions response missing id".to_string(),
            details: ProviderDetails {
                provider: Some(provider_name.to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?
        .to_string();

    let model = raw_response
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "chat completions response missing model".to_string(),
            details: ProviderDetails {
                provider: Some(provider_name.to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?
        .to_string();

    let choice = raw_response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "chat completions response missing choices[0]".to_string(),
            details: ProviderDetails {
                provider: Some(provider_name.to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?;

    let message = choice
        .get("message")
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "chat completions response missing choices[0].message".to_string(),
            details: ProviderDetails {
                provider: Some(provider_name.to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?;

    let mut content_parts = Vec::new();

    if let Some(content) = message.get("content") {
        match content {
            Value::String(text) => {
                if !text.is_empty() {
                    content_parts.push(ContentPart::Text { text: text.clone() });
                }
            }
            Value::Array(parts) => {
                for part in parts {
                    translate_message_part(part, &mut content_parts);
                }
            }
            _ => {}
        }
    }

    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for tool_call in tool_calls {
            if let Some(call) = parse_tool_call(tool_call) {
                content_parts.push(ContentPart::ToolCall { tool_call: call });
            }
        }
    }

    let raw_finish = choice.get("finish_reason").and_then(Value::as_str);
    let reason = if content_parts
        .iter()
        .any(|part| matches!(part, ContentPart::ToolCall { .. }))
    {
        Reason::ToolCalls
    } else {
        map_finish_reason(raw_finish)
    };

    let usage = parse_usage(raw_response.get("usage"));

    Ok(Response {
        id,
        model,
        provider: provider_name.to_string(),
        message: Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason {
            reason,
            raw: raw_finish.map(ToString::to_string),
        },
        usage,
        raw: Some(raw_response),
        warnings: None,
        rate_limit: headers.and_then(crate::http::headers::parse_rate_limit_headers),
    })
}

fn translate_message_part(part: &Value, content_parts: &mut Vec<ContentPart>) {
    let part_type = part.get("type").and_then(Value::as_str);
    if let Some("text" | "output_text") = part_type
        && let Some(text) = part.get("text").and_then(Value::as_str)
    {
        content_parts.push(ContentPart::Text {
            text: text.to_string(),
        });
    }
}

fn parse_tool_call(value: &Value) -> Option<ToolCallData> {
    let id = value.get("id").and_then(Value::as_str)?.to_string();
    let name = value
        .pointer("/function/name")
        .and_then(Value::as_str)?
        .to_string();
    let raw_arguments = value
        .pointer("/function/arguments")
        .and_then(Value::as_str)
        .unwrap_or("{}");

    let arguments = serde_json::from_str::<Value>(raw_arguments)
        .unwrap_or_else(|_| Value::String(raw_arguments.to_string()));

    Some(ToolCallData {
        id,
        name,
        arguments,
        call_type: "function".to_string(),
    })
}

/// Parse usage data from a Chat Completions response.
pub(crate) fn parse_usage(usage: Option<&Value>) -> Usage {
    let Some(usage) = usage else {
        return Usage::default();
    };

    let input_tokens = usage
        .get("prompt_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = usage
        .get("completion_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_tokens = usage
        .get("total_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(input_tokens + output_tokens);

    let reasoning_tokens = usage
        .pointer("/completion_tokens_details/reasoning_tokens")
        .and_then(Value::as_u64)
        .or_else(|| {
            usage
                .pointer("/output_tokens_details/reasoning_tokens")
                .and_then(Value::as_u64)
        });

    let cache_read_tokens = usage
        .pointer("/prompt_tokens_details/cached_tokens")
        .and_then(Value::as_u64);

    Usage {
        input_tokens,
        output_tokens,
        total_tokens,
        reasoning_tokens,
        cache_read_tokens,
        cache_write_tokens: None,
        raw: Some(usage.clone()),
    }
}

/// Map a raw Chat Completions finish reason string to a unified reason.
pub(crate) fn map_finish_reason(raw: Option<&str>) -> Reason {
    match raw {
        Some("stop") | None => Reason::Stop,
        Some("length") => Reason::Length,
        Some("tool_calls" | "function_call") => Reason::ToolCalls,
        Some("content_filter") => Reason::ContentFilter,
        Some("error") => Reason::Error,
        Some(_) => Reason::Other,
    }
}

// ─── Stream Translation ──────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct ToolCallState {
    id: String,
    name: String,
    arguments: String,
    started: bool,
}

/// Stateful translator for Chat Completions SSE events.
#[derive(Debug, Clone, Default)]
pub(crate) struct ChatCompletionsStreamState {
    emitted_stream_start: bool,
    emitted_text_start: bool,
    emitted_reasoning_start: bool,
    tool_calls: BTreeMap<u64, ToolCallState>,
    pending_usage: Option<Usage>,
    finished: bool,
    seen_finish_text_ids: HashSet<String>,
    rate_limit: Option<RateLimitInfo>,
    accumulated_text: String,
    accumulated_reasoning: String,
    response_id: Option<String>,
    response_model: Option<String>,
    provider_name: String,
    error_config: Option<ErrorConfig>,
}

impl ChatCompletionsStreamState {
    #[must_use]
    pub(crate) fn new(
        provider_name: &str,
        rate_limit: Option<RateLimitInfo>,
        error_config: ErrorConfig,
    ) -> Self {
        Self {
            rate_limit,
            provider_name: provider_name.to_string(),
            error_config: Some(error_config),
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
pub(crate) fn translate_sse_event(
    event: &SseEvent,
    state: &mut ChatCompletionsStreamState,
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

        // Handle reasoning_content (DeepSeek) or reasoning (OpenAI o-series
        // via Chat Completions).  The field carries chain-of-thought text.
        let reasoning_text = delta
            .get("reasoning_content")
            .and_then(Value::as_str)
            .or_else(|| delta.get("reasoning").and_then(Value::as_str));
        if let Some(reasoning_delta) = reasoning_text
            && !reasoning_delta.is_empty()
        {
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
            state.accumulated_reasoning.push_str(reasoning_delta);
            out.push(StreamEvent {
                event_type: StreamEventType::ReasoningDelta,
                delta: None,
                text_id: Some("reasoning_0".to_string()),
                reasoning_delta: Some(reasoning_delta.to_string()),
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
                    raw: Some(payload.clone()),
                });
            }

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

            if reason == Reason::ToolCalls {
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
                let reasoning = std::mem::take(&mut state.accumulated_reasoning);
                if !reasoning.is_empty() {
                    content.push(ContentPart::Thinking {
                        thinking: crate::types::content::ThinkingData {
                            text: reasoning,
                            signature: None,
                            redacted: false,
                        },
                    });
                }
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
                    provider: state.provider_name.clone(),
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
            let err = SdkError::Server {
                message,
                details: ProviderDetails {
                    provider: Some(state.provider_name.clone()),
                    raw: Some(payload.clone()),
                    retryable: true,
                    ..ProviderDetails::default()
                },
            };
            let translated = if let Some(config) = &state.error_config {
                common_translate(err, config)
            } else {
                err
            };
            state.finished = true;
            out.push(StreamEvent::error(translated));
        } else {
            out.push(provider_event(payload));
        }
    }

    Ok(out)
}

impl crate::providers::common::stream::SseStreamState for ChatCompletionsStreamState {
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
pub(crate) fn translate_sse_stream<'a>(
    sse_stream: std::pin::Pin<Box<dyn futures::Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
    state: ChatCompletionsStreamState,
) -> crate::provider::BoxStream<'a, SdkResult<StreamEvent>> {
    crate::providers::common::stream::translate_sse_stream(sse_stream, state)
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
