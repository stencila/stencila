use serde_json::Value;

use crate::error::SdkResult;
use crate::types::content::{ContentPart, ThinkingData, ToolCallData};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::usage::Usage;

/// Translate a Gemini API response body into a unified response.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when required response fields are missing.
pub fn translate_response(
    raw_response: Value,
    rate_limit: Option<RateLimitInfo>,
) -> SdkResult<Response> {
    let model = raw_response
        .get("modelVersion")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    // Gemini does not return a top-level id; synthesize one.
    let id = format!("gemini_{}", uuid::Uuid::new_v4());

    let content = translate_candidates(&raw_response);
    let finish_reason = extract_finish_reason(&raw_response, &content);
    let usage = parse_usage(raw_response.get("usageMetadata"));

    Ok(Response {
        id,
        model,
        provider: "gemini".to_string(),
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

fn translate_candidates(raw_response: &Value) -> Vec<ContentPart> {
    let mut content = Vec::new();

    let Some(candidates) = raw_response.get("candidates").and_then(Value::as_array) else {
        return content;
    };

    // Use the first candidate (index 0)
    let Some(candidate) = candidates.first() else {
        return content;
    };

    if let Some(parts) = candidate
        .pointer("/content/parts")
        .and_then(Value::as_array)
    {
        for part in parts {
            translate_candidate_part(part, &mut content);
        }
    }

    content
}

fn translate_candidate_part(part: &Value, content: &mut Vec<ContentPart>) {
    // Text part
    if let Some(text) = part.get("text").and_then(Value::as_str) {
        // Check if this is a thinking/thought part (Gemini 2.5)
        if part
            .get("thought")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            content.push(ContentPart::Thinking {
                thinking: ThinkingData {
                    text: text.to_string(),
                    signature: None,
                    redacted: false,
                },
            });
        } else {
            content.push(ContentPart::Text {
                text: text.to_string(),
            });
        }
        return;
    }

    // Function call part
    if let Some(function_call) = part.get("functionCall") {
        let name = function_call
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown_function")
            .to_string();

        let args = function_call
            .get("args")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        let id = format!("call_{}", uuid::Uuid::new_v4());

        content.push(ContentPart::ToolCall {
            tool_call: ToolCallData {
                id,
                name,
                arguments: args,
                call_type: "function".to_string(),
            },
        });
    }
}

fn extract_finish_reason(raw_response: &Value, content: &[ContentPart]) -> FinishReason {
    let raw = raw_response
        .pointer("/candidates/0/finishReason")
        .and_then(Value::as_str)
        .map(ToString::to_string);

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
        Some("STOP") | None => Reason::Stop,
        Some("MAX_TOKENS") => Reason::Length,
        Some("SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" | "SPII") => {
            Reason::ContentFilter
        }
        Some("MALFORMED_FUNCTION_CALL") => Reason::Error,
        Some(_) => Reason::Other,
    }
}

pub(crate) fn parse_usage(usage: Option<&Value>) -> Usage {
    let Some(usage) = usage else {
        return Usage::default();
    };

    let input_tokens = usage
        .get("promptTokenCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let output_tokens = usage
        .get("candidatesTokenCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let total_tokens = usage
        .get("totalTokenCount")
        .and_then(Value::as_u64)
        .unwrap_or(input_tokens + output_tokens);

    let cache_read_tokens = usage.get("cachedContentTokenCount").and_then(Value::as_u64);

    let reasoning_tokens = usage.get("thoughtsTokenCount").and_then(Value::as_u64);

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
