use serde_json::Value;

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::types::content::{ContentPart, ThinkingData, ToolCallData};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::usage::Usage;

/// Translate an Anthropic Messages API response body into a unified response.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when required response fields are missing.
pub fn translate_response(
    raw_response: Value,
    rate_limit: Option<RateLimitInfo>,
) -> SdkResult<Response> {
    let id = raw_response
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "Anthropic response missing id".to_string(),
            details: ProviderDetails {
                provider: Some("anthropic".to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?
        .to_string();

    let model = raw_response
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "Anthropic response missing model".to_string(),
            details: ProviderDetails {
                provider: Some("anthropic".to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?
        .to_string();

    let content = translate_content_blocks(&raw_response);
    let finish_reason = extract_finish_reason(&raw_response, &content);
    let mut usage = parse_usage(raw_response.get("usage"));
    usage.reasoning_tokens = estimate_reasoning_tokens_from_content(&content);

    Ok(Response {
        id,
        model,
        provider: "anthropic".to_string(),
        message: Message {
            role: Role::Assistant,
            content,
            name: None,
            tool_call_id: None,
        },
        finish_reason,
        usage,
        raw: Some(raw_response),
        warnings: None,
        rate_limit,
    })
}

fn translate_content_blocks(raw_response: &Value) -> Vec<ContentPart> {
    let mut content = Vec::new();

    let Some(blocks) = raw_response.get("content").and_then(Value::as_array) else {
        return content;
    };

    for block in blocks {
        let Some(block_type) = block.get("type").and_then(Value::as_str) else {
            continue;
        };

        match block_type {
            "text" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    content.push(ContentPart::Text {
                        text: text.to_string(),
                    });
                }
            }
            "tool_use" => {
                if let Some(tool_call) = parse_tool_call(block) {
                    content.push(ContentPart::ToolCall { tool_call });
                }
            }
            "thinking" => {
                let text = block
                    .get("thinking")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let signature = block
                    .get("signature")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                content.push(ContentPart::Thinking {
                    thinking: ThinkingData {
                        text,
                        signature,
                        redacted: false,
                    },
                });
            }
            "redacted_thinking" => {
                let data = block
                    .get("data")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                content.push(ContentPart::RedactedThinking {
                    thinking: ThinkingData {
                        text: data,
                        signature: None,
                        redacted: true,
                    },
                });
            }
            _ => {
                // Ignore unknown block types for forward compatibility
            }
        }
    }

    content
}

fn parse_tool_call(block: &Value) -> Option<ToolCallData> {
    let id = block.get("id").and_then(Value::as_str)?.to_string();
    let name = block.get("name").and_then(Value::as_str)?.to_string();
    let arguments = block
        .get("input")
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));

    Some(ToolCallData {
        id,
        name,
        arguments,
        call_type: "function".to_string(),
        thought_signature: None,
    })
}

pub(crate) fn parse_usage(usage: Option<&Value>) -> Usage {
    let Some(usage) = usage else {
        return Usage::default();
    };

    let input_tokens = usage
        .get("input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let output_tokens = usage
        .get("output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let total_tokens = input_tokens + output_tokens;

    let cache_read_tokens = usage.get("cache_read_input_tokens").and_then(Value::as_u64);

    let cache_write_tokens = usage
        .get("cache_creation_input_tokens")
        .and_then(Value::as_u64);

    Usage {
        input_tokens,
        output_tokens,
        total_tokens,
        reasoning_tokens: None,
        cache_read_tokens,
        cache_write_tokens,
        raw: Some(usage.clone()),
    }
}

/// Estimate Anthropic reasoning tokens from thinking content blocks.
///
/// Anthropic does not provide a dedicated reasoning-token usage field.
/// We estimate by counting whitespace-delimited tokens in thinking blocks.
#[must_use]
pub(crate) fn estimate_reasoning_tokens_from_content(content: &[ContentPart]) -> Option<u64> {
    let total: u64 = content
        .iter()
        .filter_map(|part| match part {
            ContentPart::Thinking { thinking } | ContentPart::RedactedThinking { thinking } => {
                Some(estimate_reasoning_tokens_from_text(&thinking.text))
            }
            _ => None,
        })
        .sum();

    if total == 0 { None } else { Some(total) }
}

#[must_use]
fn estimate_reasoning_tokens_from_text(text: &str) -> u64 {
    text.split_whitespace()
        .filter(|chunk| !chunk.is_empty())
        .count() as u64
}

fn extract_finish_reason(raw_response: &Value, content: &[ContentPart]) -> FinishReason {
    let raw = raw_response
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    // If content contains tool calls, override to ToolCalls
    let reason = if content
        .iter()
        .any(|part| matches!(part, ContentPart::ToolCall { .. }))
    {
        Reason::ToolCalls
    } else {
        map_finish_reason(raw.as_deref())
    };

    FinishReason { reason, raw }
}

pub(crate) fn map_finish_reason(raw: Option<&str>) -> Reason {
    match raw {
        Some("end_turn" | "stop" | "stop_sequence") | None => Reason::Stop,
        Some("max_tokens") => Reason::Length,
        Some("tool_use") => Reason::ToolCalls,
        Some("content_filter") => Reason::ContentFilter,
        Some(_) => Reason::Other,
    }
}
