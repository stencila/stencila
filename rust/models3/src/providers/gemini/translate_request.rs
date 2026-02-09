use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::providers::common::image::read_local_image_from_url;
use crate::types::content::ContentPart;
use crate::types::message::Message;
use crate::types::request::Request;
use crate::types::response_format::ResponseFormatType;
use crate::types::role::Role;
use crate::types::tool::{ToolChoice, ToolDefinition};

/// Translate a unified request into a Gemini API request body.
///
/// Gemini authentication is handled via query parameter, so this function
/// only returns the JSON body (no separate headers needed).
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when the request contains unsupported
/// content for Gemini translation, or invalid provider options.
pub fn translate_request(request: &Request) -> SdkResult<Value> {
    let mut body = Map::new();

    let mut system_parts: Vec<Value> = Vec::new();
    let mut contents: Vec<Value> = Vec::new();

    for message in &request.messages {
        translate_message(message, &request.messages, &mut system_parts, &mut contents)?;
    }

    if !system_parts.is_empty() {
        body.insert(
            "systemInstruction".to_string(),
            json!({ "parts": system_parts }),
        );
    }

    body.insert("contents".to_string(), Value::Array(contents));

    // Generation config
    let mut gen_config = Map::new();
    if let Some(temperature) = request.temperature {
        gen_config.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(top_p) = request.top_p {
        gen_config.insert("topP".to_string(), json!(top_p));
    }
    if let Some(max_tokens) = request.max_tokens {
        gen_config.insert("maxOutputTokens".to_string(), json!(max_tokens));
    }
    if let Some(stop_sequences) = &request.stop_sequences {
        gen_config.insert("stopSequences".to_string(), json!(stop_sequences));
    }

    if let Some(response_format) = &request.response_format {
        apply_response_format(response_format, &mut gen_config);
    }

    if !gen_config.is_empty() {
        body.insert("generationConfig".to_string(), Value::Object(gen_config));
    }

    // Tools
    if let Some(tools) = &request.tools {
        let declarations: SdkResult<Vec<Value>> =
            tools.iter().map(translate_tool_definition).collect();
        let declarations = declarations?;
        if !declarations.is_empty() {
            body.insert(
                "tools".to_string(),
                json!([{ "functionDeclarations": declarations }]),
            );
        }
    }

    // Tool choice
    if let Some(tool_choice) = &request.tool_choice {
        body.insert(
            "toolConfig".to_string(),
            json!({ "functionCallingConfig": translate_tool_choice(tool_choice) }),
        );
    }

    // Provider-specific options
    if let Some(options) = request.provider_options_for("gemini") {
        apply_provider_options(options, &mut body)?;
    }

    Ok(Value::Object(body))
}

#[allow(clippy::too_many_lines)]
fn translate_message(
    message: &Message,
    all_messages: &[Message],
    system_parts: &mut Vec<Value>,
    contents: &mut Vec<Value>,
) -> SdkResult<()> {
    match message.role {
        Role::System | Role::Developer => {
            for part in &message.content {
                match part {
                    ContentPart::Text { text } => {
                        system_parts.push(json!({ "text": text }));
                    }
                    _ => {
                        return Err(SdkError::InvalidRequest {
                            message: "Gemini system instructions only support text content"
                                .to_string(),
                            details: ProviderDetails {
                                provider: Some("gemini".to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }
        }
        Role::User => {
            let parts = translate_content_parts(&message.content, all_messages)?;
            if !parts.is_empty() {
                contents.push(json!({ "role": "user", "parts": parts }));
            }
        }
        Role::Assistant => {
            let parts = translate_content_parts(&message.content, all_messages)?;
            if !parts.is_empty() {
                contents.push(json!({ "role": "model", "parts": parts }));
            }
        }
        Role::Tool => {
            let mut parts = Vec::new();
            for part in &message.content {
                match part {
                    ContentPart::ToolResult { tool_result } => {
                        let function_name = find_function_name(
                            all_messages,
                            &tool_result.tool_call_id,
                        )
                        .ok_or_else(|| SdkError::InvalidRequest {
                            message: format!(
                                "Gemini adapter: no function name found for tool_call_id '{}'",
                                tool_result.tool_call_id
                            ),
                            details: ProviderDetails {
                                provider: Some("gemini".to_string()),
                                ..ProviderDetails::default()
                            },
                        })?;

                        let response = wrap_tool_result_content(&tool_result.content);
                        parts.push(json!({
                            "functionResponse": {
                                "name": function_name,
                                "response": response
                            }
                        }));
                    }
                    ContentPart::Text { text } => {
                        let tool_call_id = message.tool_call_id.clone().ok_or_else(|| {
                            SdkError::InvalidRequest {
                                message: "tool-role text messages require tool_call_id".to_string(),
                                details: ProviderDetails {
                                    provider: Some("gemini".to_string()),
                                    ..ProviderDetails::default()
                                },
                            }
                        })?;
                        let function_name = find_function_name(all_messages, &tool_call_id)
                            .ok_or_else(|| SdkError::InvalidRequest {
                            message: format!(
                                "Gemini adapter: no function name found for tool_call_id '{tool_call_id}'"
                            ),
                            details: ProviderDetails {
                                provider: Some("gemini".to_string()),
                                ..ProviderDetails::default()
                            },
                        })?;

                        parts.push(json!({
                            "functionResponse": {
                                "name": function_name,
                                "response": { "result": text }
                            }
                        }));
                    }
                    _ => {
                        return Err(SdkError::InvalidRequest {
                            message: format!(
                                "tool-role messages only support tool results or text in Gemini adapter: {part:?}"
                            ),
                            details: ProviderDetails {
                                provider: Some("gemini".to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }
            if !parts.is_empty() {
                contents.push(json!({ "role": "user", "parts": parts }));
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn translate_content_parts(
    parts: &[ContentPart],
    all_messages: &[Message],
) -> SdkResult<Vec<Value>> {
    let mut result = Vec::new();

    for part in parts {
        match part {
            ContentPart::Text { text } => {
                result.push(json!({ "text": text }));
            }
            ContentPart::Image { image } => {
                image.validate()?;
                if let Some(url) = &image.url {
                    if let Some((data, media_type)) =
                        read_local_image_from_url(url, image.media_type.as_deref(), "gemini")?
                    {
                        use base64::Engine;
                        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
                        result.push(json!({
                            "inlineData": {
                                "mimeType": media_type,
                                "data": encoded
                            }
                        }));
                    } else {
                        let media_type = image.effective_media_type().unwrap_or("image/png");
                        result.push(json!({
                            "fileData": {
                                "mimeType": media_type,
                                "fileUri": url
                            }
                        }));
                    }
                } else if let Some(data) = &image.data {
                    use base64::Engine;
                    let media_type = image.effective_media_type().unwrap_or("image/png");
                    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
                    result.push(json!({
                        "inlineData": {
                            "mimeType": media_type,
                            "data": encoded
                        }
                    }));
                }
            }
            ContentPart::ToolCall { tool_call } => {
                let args = if tool_call.arguments.is_string() {
                    let raw = tool_call.arguments.as_str().unwrap_or("{}");
                    serde_json::from_str::<Value>(raw).map_err(|e| SdkError::InvalidRequest {
                        message: format!(
                            "Gemini tool call '{}': malformed JSON arguments: {e}",
                            tool_call.name
                        ),
                        details: ProviderDetails {
                            provider: Some("gemini".to_string()),
                            ..ProviderDetails::default()
                        },
                    })?
                } else {
                    tool_call.arguments.clone()
                };

                result.push(json!({
                    "functionCall": {
                        "name": tool_call.name,
                        "args": args
                    }
                }));
            }
            ContentPart::ToolResult { tool_result } => {
                let function_name = find_function_name(all_messages, &tool_result.tool_call_id)
                    .ok_or_else(|| SdkError::InvalidRequest {
                        message: format!(
                            "Gemini adapter: no function name found for tool_call_id '{}'",
                            tool_result.tool_call_id
                        ),
                        details: ProviderDetails {
                            provider: Some("gemini".to_string()),
                            ..ProviderDetails::default()
                        },
                    })?;

                let response = wrap_tool_result_content(&tool_result.content);
                result.push(json!({
                    "functionResponse": {
                        "name": function_name,
                        "response": response
                    }
                }));
            }
            ContentPart::Audio { .. } => {
                return Err(SdkError::InvalidRequest {
                    message: "Gemini adapter does not support audio content parts".to_string(),
                    details: ProviderDetails {
                        provider: Some("gemini".to_string()),
                        ..ProviderDetails::default()
                    },
                });
            }
            ContentPart::Document { document } => {
                document.validate()?;
                if let Some(data) = &document.data {
                    use base64::Engine;
                    let media_type = document.media_type.as_deref().unwrap_or("application/pdf");
                    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
                    result.push(json!({
                        "inlineData": {
                            "mimeType": media_type,
                            "data": encoded
                        }
                    }));
                } else if let Some(url) = &document.url {
                    let media_type = document.media_type.as_deref().unwrap_or("application/pdf");
                    result.push(json!({
                        "fileData": {
                            "mimeType": media_type,
                            "fileUri": url
                        }
                    }));
                }
            }
            // Thinking, RedactedThinking, and unknown extensions: skip on input
            ContentPart::Thinking { .. }
            | ContentPart::RedactedThinking { .. }
            | ContentPart::Extension(_) => {}
        }
    }

    Ok(result)
}

/// Wrap a tool result content value for Gemini's `functionResponse.response` field.
///
/// Gemini expects the response to be a JSON object. String results are wrapped
/// as `{"result": "the string"}`.
fn wrap_tool_result_content(content: &Value) -> Value {
    if content.is_object() {
        content.clone()
    } else if content.is_array() {
        json!({ "result": content })
    } else if let Some(s) = content.as_str() {
        json!({ "result": s })
    } else {
        json!({ "result": content })
    }
}

/// Look up the function name for a given `tool_call_id` by searching
/// previous assistant messages for a matching `ToolCall`.
///
/// Gemini uses function names (not call IDs) to correlate tool responses
/// with their originating calls.
fn find_function_name(messages: &[Message], tool_call_id: &str) -> Option<String> {
    for msg in messages.iter().rev() {
        if msg.role == Role::Assistant {
            for part in &msg.content {
                if let ContentPart::ToolCall { tool_call } = part
                    && tool_call.id == tool_call_id
                {
                    return Some(tool_call.name.clone());
                }
            }
        }
    }
    None
}

fn translate_tool_definition(tool: &ToolDefinition) -> SdkResult<Value> {
    tool.validate()?;

    let mut decl = Map::new();
    decl.insert("name".to_string(), Value::String(tool.name.clone()));
    decl.insert(
        "description".to_string(),
        Value::String(tool.description.clone()),
    );
    decl.insert("parameters".to_string(), tool.parameters.clone());

    Ok(Value::Object(decl))
}

fn translate_tool_choice(tool_choice: &ToolChoice) -> Value {
    match tool_choice {
        ToolChoice::Auto => json!({ "mode": "AUTO" }),
        ToolChoice::None => json!({ "mode": "NONE" }),
        ToolChoice::Required => json!({ "mode": "ANY" }),
        ToolChoice::Tool(name) => json!({
            "mode": "ANY",
            "allowedFunctionNames": [name]
        }),
    }
}

fn apply_response_format(
    format: &crate::types::response_format::ResponseFormat,
    gen_config: &mut Map<String, Value>,
) {
    match format.format_type {
        ResponseFormatType::Text => {
            gen_config.insert(
                "responseMimeType".to_string(),
                Value::String("text/plain".to_string()),
            );
        }
        ResponseFormatType::Json => {
            gen_config.insert(
                "responseMimeType".to_string(),
                Value::String("application/json".to_string()),
            );
        }
        ResponseFormatType::JsonSchema => {
            gen_config.insert(
                "responseMimeType".to_string(),
                Value::String("application/json".to_string()),
            );
            if let Some(schema) = &format.json_schema {
                gen_config.insert("responseSchema".to_string(), schema.clone());
            }
        }
    }
}

fn apply_provider_options(options: &Value, body: &mut Map<String, Value>) -> SdkResult<()> {
    let Some(options_obj) = options.as_object() else {
        return Err(SdkError::InvalidRequest {
            message: "provider_options.gemini must be an object".to_string(),
            details: ProviderDetails {
                provider: Some("gemini".to_string()),
                ..ProviderDetails::default()
            },
        });
    };

    for (key, value) in options_obj {
        body.insert(key.clone(), value.clone());
    }

    Ok(())
}
