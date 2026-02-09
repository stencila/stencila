use reqwest::header::HeaderMap;
use serde_json::Value;

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::headers::parse_rate_limit_headers;
use crate::types::content::{ContentPart, ToolCallData};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::usage::Usage;

/// Translate a Chat Completions response into a unified response.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` if required fields are missing.
pub fn translate_response(raw_response: Value, headers: Option<&HeaderMap>) -> SdkResult<Response> {
    let id = raw_response
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "chat completions response missing id".to_string(),
            details: ProviderDetails {
                provider: Some("openai_chat_completions".to_string()),
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
                provider: Some("openai_chat_completions".to_string()),
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
                provider: Some("openai_chat_completions".to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?;

    let message = choice
        .get("message")
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "chat completions response missing choices[0].message".to_string(),
            details: ProviderDetails {
                provider: Some("openai_chat_completions".to_string()),
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
        provider: "openai_chat_completions".to_string(),
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
        rate_limit: headers.and_then(parse_rate_limit_headers),
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

    let cache_read_tokens = usage
        .pointer("/prompt_tokens_details/cached_tokens")
        .and_then(Value::as_u64);

    Usage {
        input_tokens,
        output_tokens,
        total_tokens,
        reasoning_tokens: None,
        cache_read_tokens,
        cache_write_tokens: None,
        raw: Some(usage.clone()),
    }
}

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
