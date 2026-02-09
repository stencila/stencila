use reqwest::header::HeaderMap;
use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::providers::common::openai_shared::{
    image_to_openai_url, parse_custom_headers, translate_response_format, translate_tool_choice,
    value_to_json_string,
};
use crate::types::content::ContentPart;
use crate::types::request::Request;
use crate::types::role::Role;
use crate::types::tool::ToolDefinition;

/// Chat Completions translated request body + per-request headers.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedChatCompletionsRequest {
    pub body: Value,
    pub headers: HeaderMap,
}

/// Translate a unified request into an OpenAI-compatible Chat Completions request.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` for unsupported content or invalid
/// provider options.
pub fn translate_request(
    request: &Request,
    stream: bool,
) -> SdkResult<TranslatedChatCompletionsRequest> {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(request.model.clone()));

    let mut messages = Vec::new();
    for message in &request.messages {
        translate_message(message, &mut messages)?;
    }
    body.insert("messages".to_string(), Value::Array(messages));

    if let Some(tools) = &request.tools {
        let translated_tools: SdkResult<Vec<Value>> =
            tools.iter().map(translate_tool_definition).collect();
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
    if let Some(options) = request
        .provider_options_for("openai_chat_completions")
        .or_else(|| request.provider_options_for("openai_compatible"))
    {
        apply_provider_options(options, &mut body, &mut headers)?;
    }

    // Guardrail: this adapter intentionally does not support Responses-only features.
    // Check all provider option namespaces that could carry built-in tools.
    let builtin_tools_keys = ["built_in_tools", "builtin_tools"];
    for ns in ["openai", "openai_chat_completions", "openai_compatible"] {
        if let Some(options) = request.provider_options_for(ns)
            && builtin_tools_keys.iter().any(|k| options.get(*k).is_some())
        {
            return Err(SdkError::InvalidRequest {
                message:
                    "OpenAI Chat Completions adapter does not support built-in tools (Responses-only)"
                        .to_string(),
                details: ProviderDetails {
                    provider: Some("openai_chat_completions".to_string()),
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
                            provider: Some("openai_chat_completions".to_string()),
                            ..ProviderDetails::default()
                        },
                    }),
                })
                .collect::<SdkResult<Vec<String>>>()?
                .join("\n\n");

            messages.push(json!({"role": "system", "content": text}));
        }
        Role::User => {
            let content = translate_user_content_parts(&message.content)?;
            messages.push(json!({"role": "user", "content": content}));
        }
        Role::Assistant => {
            let mut text_parts = Vec::new();
            let mut tool_calls = Vec::new();

            for part in &message.content {
                match part {
                    ContentPart::Text { text } => text_parts.push(text.clone()),
                    ContentPart::ToolCall { tool_call } => {
                        tool_calls.push(json!({
                            "id": tool_call.id,
                            "type": "function",
                            "function": {
                                "name": tool_call.name,
                                "arguments": value_to_json_string(&tool_call.arguments, "openai_chat_completions")?
                            }
                        }));
                    }
                    ContentPart::Thinking { .. }
                    | ContentPart::RedactedThinking { .. }
                    | ContentPart::Audio { .. }
                    | ContentPart::Document { .. }
                    | ContentPart::Image { .. }
                    | ContentPart::ToolResult { .. }
                    | ContentPart::Extension(_) => {
                        return Err(SdkError::InvalidRequest {
                            message: format!(
                                "unsupported assistant content in Chat Completions translation: {part:?}"
                            ),
                            details: ProviderDetails {
                                provider: Some("openai_chat_completions".to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }

            let mut assistant_message = Map::new();
            assistant_message.insert("role".to_string(), Value::String("assistant".to_string()));

            if text_parts.is_empty() {
                assistant_message.insert("content".to_string(), Value::Null);
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
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_result.tool_call_id,
                            "content": tool_result.content
                        }));
                    }
                    ContentPart::Text { text } => {
                        let tool_call_id = message.tool_call_id.clone().ok_or_else(|| {
                            SdkError::InvalidRequest {
                                message: "tool-role text messages require tool_call_id".to_string(),
                                details: ProviderDetails {
                                    provider: Some("openai_chat_completions".to_string()),
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
                                provider: Some("openai_chat_completions".to_string()),
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

fn translate_user_content_parts(parts: &[ContentPart]) -> SdkResult<Value> {
    let mut items = Vec::new();
    for part in parts {
        match part {
            ContentPart::Text { text } => {
                items.push(json!({"type": "text", "text": text}));
            }
            ContentPart::Image { image } => {
                image.validate()?;
                items.push(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": image_to_openai_url(image, "openai_chat_completions")?,
                        "detail": image.detail
                    }
                }));
            }
            _ => {
                return Err(SdkError::InvalidRequest {
                    message: format!(
                        "unsupported user content in Chat Completions translation: {part:?}"
                    ),
                    details: ProviderDetails {
                        provider: Some("openai_chat_completions".to_string()),
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

fn translate_tool_definition(tool: &ToolDefinition) -> SdkResult<Value> {
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
) -> SdkResult<()> {
    let Some(options_obj) = options.as_object() else {
        return Err(SdkError::InvalidRequest {
            message: "provider_options.openai_chat_completions must be an object".to_string(),
            details: ProviderDetails {
                provider: Some("openai_chat_completions".to_string()),
                ..ProviderDetails::default()
            },
        });
    };

    parse_custom_headers(options_obj, headers, "openai_chat_completions")?;

    for (key, value) in options_obj {
        if matches!(key.as_str(), "custom_headers" | "headers") {
            continue;
        }
        body.insert(key.clone(), value.clone());
    }

    Ok(())
}
