use reqwest::header::HeaderMap;
use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::http::headers::parse_rate_limit_headers;
use crate::types::content::{ContentPart, ThinkingData, ToolCallData};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::usage::Usage;

/// Translate an OpenAI Responses API body into a unified response.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when required response fields are missing.
pub fn translate_response(raw_response: Value, headers: Option<&HeaderMap>) -> SdkResult<Response> {
    let id = raw_response
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "OpenAI response missing id".to_string(),
            details: ProviderDetails {
                provider: Some("openai".to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?
        .to_string();

    let model = raw_response
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "OpenAI response missing model".to_string(),
            details: ProviderDetails {
                provider: Some("openai".to_string()),
                raw: Some(raw_response.clone()),
                ..ProviderDetails::default()
            },
        })?
        .to_string();

    let content = translate_output_content(&raw_response);
    let finish_reason = extract_finish_reason(&raw_response, &content);

    let usage = parse_usage(raw_response.get("usage"));

    let rate_limit = headers.and_then(parse_rate_limit_headers);

    Ok(Response {
        id,
        model,
        provider: "openai".to_string(),
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

fn translate_output_content(raw_response: &Value) -> Vec<ContentPart> {
    let mut content = Vec::new();

    if let Some(output) = raw_response.get("output").and_then(Value::as_array) {
        for item in output {
            translate_output_item(item, &mut content);
        }
    }

    if content.is_empty()
        && let Some(output_text) = raw_response.get("output_text").and_then(Value::as_str)
    {
        content.push(ContentPart::Text {
            text: output_text.to_string(),
        });
    }

    content
}

fn translate_output_item(item: &Value, content: &mut Vec<ContentPart>) {
    let Some(item_type) = item.get("type").and_then(Value::as_str) else {
        return;
    };

    match item_type {
        "message" => {
            if let Some(parts) = item.get("content").and_then(Value::as_array) {
                for part in parts {
                    translate_message_content_part(part, content);
                }
            }
        }
        "function_call" | "custom_tool_call" | "local_shell_call" => {
            if let Some(tool_call) = parse_tool_call(item) {
                content.push(ContentPart::ToolCall { tool_call });
            }
        }
        "output_text" => {
            if let Some(text) = item.get("text").and_then(Value::as_str) {
                content.push(ContentPart::Text {
                    text: text.to_string(),
                });
            }
        }
        "reasoning" => {
            if let Some(text) = extract_reasoning_text(item) {
                content.push(ContentPart::Thinking {
                    thinking: ThinkingData {
                        text,
                        signature: None,
                        redacted: false,
                    },
                });
            }
        }
        _ => {
            // Preserve forward compatibility by ignoring unknown output item types.
        }
    }
}

/// Extract reasoning text from a reasoning output item or content part.
///
/// The OpenAI Responses API can return reasoning summaries in several formats:
/// - `"text": "..."` — plain text field
/// - `"summary": [{"type": "summary_text", "text": "..."}]` — array of summary parts
/// - `"summary": "..."` — plain string (legacy/forward-compat)
fn extract_reasoning_text(value: &Value) -> Option<String> {
    // Direct text field
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }

    // Summary as array of parts (the standard Responses API format)
    if let Some(summary_arr) = value.get("summary").and_then(Value::as_array) {
        let texts: Vec<&str> = summary_arr
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect();
        if !texts.is_empty() {
            return Some(texts.join(""));
        }
    }

    // Summary as plain string (fallback)
    if let Some(text) = value.get("summary").and_then(Value::as_str) {
        return Some(text.to_string());
    }

    None
}

fn translate_message_content_part(part: &Value, content: &mut Vec<ContentPart>) {
    let Some(part_type) = part.get("type").and_then(Value::as_str) else {
        return;
    };

    match part_type {
        "output_text" | "input_text" | "text" => {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                content.push(ContentPart::Text {
                    text: text.to_string(),
                });
            }
        }
        "function_call" => {
            if let Some(tool_call) = parse_tool_call(part) {
                content.push(ContentPart::ToolCall { tool_call });
            }
        }
        "reasoning" => {
            if let Some(text) = extract_reasoning_text(part) {
                content.push(ContentPart::Thinking {
                    thinking: ThinkingData {
                        text,
                        signature: None,
                        redacted: false,
                    },
                });
            }
        }
        _ => {}
    }
}

fn parse_tool_call(value: &Value) -> Option<ToolCallData> {
    // Upstream reference:
    // https://github.com/openai/codex/blob/main/codex-rs/protocol/src/models.rs
    // `ResponseItem` includes `function_call`, `custom_tool_call`, and
    // `local_shell_call`, each of which needs normalization into unified tools.
    let item_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let id = value
        // Prefer `call_id` over item `id`.
        //
        // In the OpenAI Responses API, function-call output items can include
        // both:
        // - `id`      (the output item identifier, often `fc_*`)
        // - `call_id` (the invocation identifier to correlate tool outputs)
        //
        // Tool-result messages must reference `call_id`, not the output item
        // `id`. Using `id` can produce invalid follow-up requests where
        // `function_call_output.call_id` does not match what the provider
        // expects.
        .get("call_id")
        .and_then(Value::as_str)
        .or_else(|| value.get("id").and_then(Value::as_str))?
        .to_string();
    let (name, arguments, call_type) = match item_type {
        "custom_tool_call" => {
            let name = value.get("name").and_then(Value::as_str)?.to_string();
            let input = value
                .get("input")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let (parsed, parse_error) = crate::types::tool::ToolCall::parse_arguments(&input);
            let arguments = if parse_error.is_none() && parsed.is_object() {
                parsed
            } else if name == "apply_patch" {
                json!({ "patch": input })
            } else {
                json!({ "input": input })
            };
            (name, arguments, "custom".to_string())
        }
        "local_shell_call" => {
            // Normalize OpenAI local-shell actions to the project's `shell` tool args.
            let action = value.get("action").and_then(Value::as_object);
            let args = local_shell_action_to_arguments(action);
            ("shell".to_string(), args, "local_shell".to_string())
        }
        _ => {
            let name = value
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| value.pointer("/function/name").and_then(Value::as_str))?
                .to_string();

            let arguments = value
                .get("arguments")
                .cloned()
                .or_else(|| value.pointer("/function/arguments").cloned())
                .unwrap_or(Value::Object(Map::new()));

            let arguments = if let Some(arguments_str) = arguments.as_str() {
                let (parsed, _) = crate::types::tool::ToolCall::parse_arguments(arguments_str);
                parsed
            } else {
                arguments
            };
            (name, arguments, "function".to_string())
        }
    };

    Some(ToolCallData {
        id,
        name,
        arguments,
        call_type,
        thought_signature: None,
    })
}

fn local_shell_action_to_arguments(action: Option<&Map<String, Value>>) -> Value {
    let Some(action) = action else {
        return Value::Object(Map::new());
    };

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

    if command.is_empty() {
        Value::Object(Map::new())
    } else {
        json!({ "command": command })
    }
}

pub(crate) fn parse_usage(usage: Option<&Value>) -> Usage {
    let Some(usage) = usage else {
        return Usage::default();
    };

    let input_tokens = usage
        .get("prompt_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("input_tokens").and_then(Value::as_u64))
        .unwrap_or(0);

    let output_tokens = usage
        .get("completion_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("output_tokens").and_then(Value::as_u64))
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
        .and_then(Value::as_u64)
        .or_else(|| {
            usage
                .pointer("/input_tokens_details/cached_tokens")
                .and_then(Value::as_u64)
        });

    let cache_write_tokens = usage
        .get("cache_creation_input_tokens")
        .and_then(Value::as_u64);

    Usage {
        input_tokens,
        output_tokens,
        total_tokens,
        reasoning_tokens,
        cache_read_tokens,
        cache_write_tokens,
        raw: Some(usage.clone()),
    }
}

fn extract_finish_reason(raw_response: &Value, content: &[ContentPart]) -> FinishReason {
    let raw = raw_response
        .get("finish_reason")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            raw_response
                .get("output")
                .and_then(Value::as_array)
                .and_then(|arr| {
                    arr.iter()
                        .find_map(|item| item.get("finish_reason").and_then(Value::as_str))
                        .map(ToString::to_string)
                })
        })
        .or_else(|| {
            raw_response
                .get("status")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });

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
        Some("stop" | "completed" | "end_turn") | None => Reason::Stop,
        Some("length" | "max_tokens" | "incomplete") => Reason::Length,
        Some("tool_calls" | "function_call") => Reason::ToolCalls,
        Some("content_filter" | "safety") => Reason::ContentFilter,
        Some("error" | "failed") => Reason::Error,
        Some(_) => Reason::Other,
    }
}
