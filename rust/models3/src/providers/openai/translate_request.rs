use reqwest::header::HeaderMap;
use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::providers::common::openai_shared::{
    image_to_openai_url, parse_custom_headers, value_to_json_string,
};
use crate::types::content::ContentPart;
use crate::types::request::Request;
use crate::types::response_format::{ResponseFormat, ResponseFormatType};
use crate::types::role::Role;
use crate::types::tool::{ToolChoice, ToolDefinition};

/// OpenAI Responses API translated request body + per-request headers.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedOpenAIRequest {
    pub body: Value,
    pub headers: HeaderMap,
}

/// Translate a unified request into an OpenAI Responses API request.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when the request contains unsupported
/// content for OpenAI Responses translation, or invalid provider options.
pub fn translate_request(request: &Request, stream: bool) -> SdkResult<TranslatedOpenAIRequest> {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(request.model.clone()));

    let mut instructions = Vec::new();
    let mut input = Vec::new();

    for message in &request.messages {
        translate_message(message, &mut instructions, &mut input)?;
    }

    if !instructions.is_empty() {
        body.insert(
            "instructions".to_string(),
            Value::String(instructions.join("\n\n")),
        );
    }

    body.insert("input".to_string(), Value::Array(input));

    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(top_p) = request.top_p {
        body.insert("top_p".to_string(), json!(top_p));
    }
    if let Some(max_tokens) = request.max_tokens {
        body.insert("max_output_tokens".to_string(), json!(max_tokens));
    }
    if let Some(stop_sequences) = &request.stop_sequences {
        body.insert("stop".to_string(), json!(stop_sequences));
    }
    if let Some(reasoning_effort) = &request.reasoning_effort {
        body.insert("reasoning".to_string(), json!({"effort": reasoning_effort}));
    }
    if let Some(metadata) = &request.metadata {
        body.insert("metadata".to_string(), json!(metadata));
    }

    if let Some(response_format) = &request.response_format {
        body.insert(
            "text".to_string(),
            json!({"format": translate_response_format(response_format)}),
        );
    }

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

    let mut headers = HeaderMap::new();

    if let Some(options) = request.provider_options_for("openai") {
        apply_provider_options(options, &mut body, &mut headers)?;
    }

    if stream {
        body.insert("stream".to_string(), Value::Bool(true));
    }

    Ok(TranslatedOpenAIRequest {
        body: Value::Object(body),
        headers,
    })
}

#[allow(clippy::too_many_lines)]
fn translate_message(
    message: &crate::types::message::Message,
    instructions: &mut Vec<String>,
    input: &mut Vec<Value>,
) -> SdkResult<()> {
    match message.role {
        Role::System | Role::Developer => {
            for part in &message.content {
                match part {
                    ContentPart::Text { text } => instructions.push(text.clone()),
                    _ => {
                        return Err(SdkError::InvalidRequest {
                            message: "OpenAI instructions only support text content".to_string(),
                            details: ProviderDetails {
                                provider: Some("openai".to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }
        }
        Role::User | Role::Assistant => {
            let mut message_content = Vec::new();

            for part in &message.content {
                match part {
                    ContentPart::Text { text } => {
                        let content_type = if message.role == Role::User {
                            "input_text"
                        } else {
                            "output_text"
                        };
                        message_content.push(json!({"type": content_type, "text": text}));
                    }
                    ContentPart::Image { image } => {
                        if message.role != Role::User {
                            return Err(SdkError::InvalidRequest {
                                message: "OpenAI image input is only supported in user messages"
                                    .to_string(),
                                details: ProviderDetails {
                                    provider: Some("openai".to_string()),
                                    ..ProviderDetails::default()
                                },
                            });
                        }

                        image.validate()?;
                        let image_url = image_to_openai_url(image, "openai")?;
                        let mut image_part = json!({"type": "input_image", "image_url": image_url});
                        if let Some(detail) = &image.detail {
                            image_part["detail"] = Value::String(detail.clone());
                        }
                        message_content.push(image_part);
                    }
                    ContentPart::ToolCall { tool_call } => {
                        input.push(json!({
                            "type": "function_call",
                            "call_id": tool_call.id,
                            "name": tool_call.name,
                            "arguments": value_to_json_string(&tool_call.arguments, "openai")?
                        }));
                    }
                    ContentPart::ToolResult { tool_result } => {
                        input.push(json!({
                            "type": "function_call_output",
                            "call_id": tool_result.tool_call_id,
                            "output": value_to_json_string(&tool_result.content, "openai")?
                        }));
                    }
                    ContentPart::Thinking { thinking } => {
                        let content_type = if message.role == Role::User {
                            "input_text"
                        } else {
                            "output_text"
                        };
                        // Cross-provider portability: when replaying Anthropic thinking
                        // blocks into OpenAI, keep text but strip signatures.
                        message_content.push(json!({"type": content_type, "text": thinking.text}));
                    }
                    ContentPart::RedactedThinking { .. } => {
                        // Opaque redacted thinking cannot be meaningfully translated.
                        // Drop it when switching providers.
                    }
                    ContentPart::Audio { .. }
                    | ContentPart::Document { .. }
                    | ContentPart::Extension(_) => {
                        return Err(SdkError::InvalidRequest {
                            message: format!(
                                "OpenAI does not support this content part in request messages: {part:?}"
                            ),
                            details: ProviderDetails {
                                provider: Some("openai".to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }

            if !message_content.is_empty() {
                let role = if message.role == Role::User {
                    "user"
                } else {
                    "assistant"
                };
                input.push(json!({
                    "type": "message",
                    "role": role,
                    "content": message_content
                }));
            }
        }
        Role::Tool => {
            for part in &message.content {
                match part {
                    ContentPart::ToolResult { tool_result } => {
                        input.push(json!({
                            "type": "function_call_output",
                            "call_id": tool_result.tool_call_id,
                            "output": value_to_json_string(&tool_result.content, "openai")?
                        }));
                    }
                    ContentPart::Text { text } => {
                        let tool_call_id = message.tool_call_id.clone().ok_or_else(|| {
                            SdkError::InvalidRequest {
                                message: "tool-role text messages require tool_call_id".to_string(),
                                details: ProviderDetails {
                                    provider: Some("openai".to_string()),
                                    ..ProviderDetails::default()
                                },
                            }
                        })?;
                        input.push(json!({
                            "type": "function_call_output",
                            "call_id": tool_call_id,
                            "output": text
                        }));
                    }
                    _ => {
                        return Err(SdkError::InvalidRequest {
                            message: format!(
                                "tool-role messages only support tool results or text in OpenAI adapter: {part:?}"
                            ),
                            details: ProviderDetails {
                                provider: Some("openai".to_string()),
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

fn translate_response_format(format: &ResponseFormat) -> Value {
    match format.format_type {
        ResponseFormatType::Text => json!({"type": "text"}),
        ResponseFormatType::Json => json!({"type": "json_object"}),
        ResponseFormatType::JsonSchema => json!({
            "type": "json_schema",
            "name": "response",
            "schema": format.json_schema.clone().unwrap_or(Value::Null),
            "strict": format.strict
        }),
    }
}

fn translate_tool_definition(tool: &ToolDefinition) -> SdkResult<Value> {
    tool.validate()?;

    let mut translated = Map::new();
    translated.insert("type".to_string(), Value::String("function".to_string()));
    translated.insert("name".to_string(), Value::String(tool.name.clone()));
    translated.insert(
        "description".to_string(),
        Value::String(tool.description.clone()),
    );
    translated.insert("parameters".to_string(), tool.parameters.clone());
    if tool.strict {
        translated.insert("strict".to_string(), Value::Bool(true));
    }

    Ok(Value::Object(translated))
}

fn translate_tool_choice(tool_choice: &ToolChoice) -> Value {
    match tool_choice {
        ToolChoice::Auto => Value::String("auto".to_string()),
        ToolChoice::None => Value::String("none".to_string()),
        ToolChoice::Required => Value::String("required".to_string()),
        ToolChoice::Tool(name) => json!({
            "type": "function",
            "name": name
        }),
    }
}

fn apply_provider_options(
    options: &Value,
    body: &mut Map<String, Value>,
    headers: &mut HeaderMap,
) -> SdkResult<()> {
    let Some(options_obj) = options.as_object() else {
        return Err(SdkError::InvalidRequest {
            message: "provider_options.openai must be an object".to_string(),
            details: ProviderDetails {
                provider: Some("openai".to_string()),
                ..ProviderDetails::default()
            },
        });
    };

    if let Some(extra_tools) = options_obj
        .get("built_in_tools")
        .or_else(|| options_obj.get("builtin_tools"))
        .or_else(|| options_obj.get("tools"))
    {
        let Some(extra_tools_arr) = extra_tools.as_array() else {
            return Err(SdkError::InvalidRequest {
                message: "provider_options.openai.built_in_tools must be an array".to_string(),
                details: ProviderDetails {
                    provider: Some("openai".to_string()),
                    ..ProviderDetails::default()
                },
            });
        };

        let entry = body
            .entry("tools".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));

        let Some(tools_array) = entry.as_array_mut() else {
            return Err(SdkError::InvalidRequest {
                message: "OpenAI request body tools field is not an array".to_string(),
                details: ProviderDetails {
                    provider: Some("openai".to_string()),
                    ..ProviderDetails::default()
                },
            });
        };

        tools_array.extend(extra_tools_arr.iter().cloned());
    }

    parse_custom_headers(options_obj, headers, "openai")?;

    for (key, value) in options_obj {
        if matches!(
            key.as_str(),
            "built_in_tools" | "builtin_tools" | "tools" | "custom_headers" | "headers"
        ) {
            continue;
        }
        body.insert(key.clone(), value.clone());
    }

    Ok(())
}
