use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::providers::common::image::read_local_image_from_url;
use crate::types::content::ContentPart;
use crate::types::message::Message;
use crate::types::request::Request;
use crate::types::role::Role;
use crate::types::tool::{ToolChoice, ToolDefinition};

/// Default `max_tokens` when none is specified in the request.
const DEFAULT_MAX_TOKENS: u64 = 4096;

/// Anthropic Messages API translated request body + per-request headers.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedAnthropicRequest {
    pub body: Value,
    pub headers: HeaderMap,
}

/// Translate a unified request into an Anthropic Messages API request.
///
/// When `system_prefix` is `Some`, its text is prepended as the first system
/// block before any system messages from the request.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when the request contains unsupported
/// content for Anthropic Messages translation, or invalid provider options.
pub fn translate_request(
    request: &Request,
    stream: bool,
    system_prefix: Option<&str>,
) -> SdkResult<TranslatedAnthropicRequest> {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(request.model.clone()));

    // max_tokens is required by Anthropic
    let max_tokens = request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
    body.insert("max_tokens".to_string(), json!(max_tokens));

    // Separate system messages from conversation messages
    let mut system_blocks = Vec::new();
    let mut conversation_messages: Vec<Value> = Vec::new();

    // Prepend system prefix if provided (e.g. for OAuth identity)
    if let Some(prefix) = system_prefix {
        system_blocks.push(json!({"type": "text", "text": prefix}));
    }

    for message in &request.messages {
        translate_message(message, &mut system_blocks, &mut conversation_messages)?;
    }

    if !system_blocks.is_empty() {
        body.insert("system".to_string(), Value::Array(system_blocks));
    }

    // Enforce strict alternation: merge consecutive same-role messages
    let merged_messages = merge_consecutive_messages(conversation_messages);
    body.insert("messages".to_string(), Value::Array(merged_messages));

    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(top_p) = request.top_p {
        body.insert("top_p".to_string(), json!(top_p));
    }
    if let Some(stop_sequences) = &request.stop_sequences {
        body.insert("stop_sequences".to_string(), json!(stop_sequences));
    }
    if let Some(metadata) = &request.metadata {
        body.insert("metadata".to_string(), json!(metadata));
    }

    // Tool definitions
    if let Some(tools) = &request.tools {
        let translated_tools: SdkResult<Vec<Value>> =
            tools.iter().map(translate_tool_definition).collect();
        let translated_tools = translated_tools?;
        if !translated_tools.is_empty() {
            body.insert("tools".to_string(), Value::Array(translated_tools));
        }
    }

    // Tool choice
    if let Some(tool_choice) = &request.tool_choice {
        match translate_tool_choice(tool_choice) {
            ToolChoiceAction::SetChoice(choice) => {
                body.insert("tool_choice".to_string(), choice);
            }
            ToolChoiceAction::OmitTools => {
                // ToolChoice::None means do not use tools at all
                body.remove("tools");
            }
        }
    }

    let mut headers = HeaderMap::new();
    let mut auto_cache = true;

    // Provider-specific options
    if let Some(options) = request.provider_options_for("anthropic") {
        apply_provider_options(options, &mut body, &mut headers, &mut auto_cache)?;
    }

    // Anthropic requires max_tokens > thinking.budget_tokens when extended
    // thinking is enabled. Auto-adjust if the current value is too low.
    if let Some(budget) = body
        .get("thinking")
        .and_then(|t| t.get("budget_tokens"))
        .and_then(Value::as_u64)
    {
        let current_max = body.get("max_tokens").and_then(Value::as_u64).unwrap_or(0);
        if current_max <= budget {
            body.insert("max_tokens".to_string(), json!(budget + max_tokens));
        }
    }

    // Cache control injection (when auto_cache is enabled)
    if auto_cache {
        inject_cache_control(&mut body);
        add_beta_header(&mut headers, "prompt-caching-2024-07-31");
    }

    if stream {
        body.insert("stream".to_string(), Value::Bool(true));
    }

    Ok(TranslatedAnthropicRequest {
        body: Value::Object(body),
        headers,
    })
}

fn translate_message(
    message: &Message,
    system_blocks: &mut Vec<Value>,
    conversation: &mut Vec<Value>,
) -> SdkResult<()> {
    match message.role {
        Role::System | Role::Developer => {
            // System and developer messages go into the system parameter
            for part in &message.content {
                match part {
                    ContentPart::Text { text } => {
                        system_blocks.push(json!({"type": "text", "text": text}));
                    }
                    _ => {
                        return Err(SdkError::InvalidRequest {
                            message: "Anthropic system messages only support text content"
                                .to_string(),
                            details: ProviderDetails {
                                provider: Some("anthropic".to_string()),
                                ..ProviderDetails::default()
                            },
                        });
                    }
                }
            }
        }
        Role::User => {
            let content = translate_content_parts(&message.content, "user")?;
            if !content.is_empty() {
                conversation.push(json!({
                    "role": "user",
                    "content": content
                }));
            }
        }
        Role::Assistant => {
            let content = translate_content_parts(&message.content, "assistant")?;
            if !content.is_empty() {
                conversation.push(json!({
                    "role": "assistant",
                    "content": content
                }));
            }
        }
        Role::Tool => {
            // Tool results become user messages with tool_result content blocks
            let content = translate_content_parts(&message.content, "tool")?;
            if !content.is_empty() {
                conversation.push(json!({
                    "role": "user",
                    "content": content
                }));
            }
        }
    }

    Ok(())
}

fn translate_content_parts(parts: &[ContentPart], role: &str) -> SdkResult<Vec<Value>> {
    let mut content = Vec::new();
    for part in parts {
        translate_single_content_part(part, role, &mut content)?;
    }
    Ok(content)
}

fn translate_single_content_part(
    part: &ContentPart,
    role: &str,
    content: &mut Vec<Value>,
) -> SdkResult<()> {
    match part {
        ContentPart::Text { text } => {
            content.push(json!({"type": "text", "text": text}));
        }
        ContentPart::Image { image } => {
            image.validate()?;
            content.push(translate_image(image)?);
        }
        ContentPart::Document { document } => {
            content.push(translate_document(document)?);
        }
        ContentPart::ToolCall { tool_call } => {
            content.push(json!({
                "type": "tool_use",
                "id": tool_call.id,
                "name": tool_call.name,
                "input": tool_call.arguments
            }));
        }
        ContentPart::ToolResult { tool_result } => {
            content.push(translate_tool_result(tool_result));
        }
        ContentPart::Thinking { thinking } => {
            let mut block = json!({"type": "thinking", "thinking": thinking.text});
            if let Some(signature) = &thinking.signature {
                block["signature"] = Value::String(signature.clone());
            }
            content.push(block);
        }
        ContentPart::RedactedThinking { thinking } => {
            content.push(json!({"type": "redacted_thinking", "data": thinking.text}));
        }
        ContentPart::Audio { .. } => {
            return Err(SdkError::InvalidRequest {
                message: "Anthropic does not support audio content".to_string(),
                details: ProviderDetails {
                    provider: Some("anthropic".to_string()),
                    ..ProviderDetails::default()
                },
            });
        }
        ContentPart::Extension(_) => {
            return Err(SdkError::InvalidRequest {
                message: format!(
                    "Anthropic does not support this content part in {role} messages: {part:?}"
                ),
                details: ProviderDetails {
                    provider: Some("anthropic".to_string()),
                    ..ProviderDetails::default()
                },
            });
        }
    }
    Ok(())
}

fn translate_image(image: &crate::types::content::ImageData) -> SdkResult<Value> {
    if let Some(url) = &image.url {
        if let Some((data, media_type)) =
            read_local_image_from_url(url, image.media_type.as_deref(), "anthropic")?
        {
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(data);
            return Ok(json!({
                "type": "image",
                "source": {"type": "base64", "media_type": media_type, "data": encoded}
            }));
        }

        Ok(json!({
            "type": "image",
            "source": {"type": "url", "url": url}
        }))
    } else if let Some(data) = &image.data {
        use base64::Engine;
        let media_type = image.effective_media_type().unwrap_or("image/png");
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        Ok(json!({
            "type": "image",
            "source": {"type": "base64", "media_type": media_type, "data": encoded}
        }))
    } else {
        Err(SdkError::InvalidRequest {
            message: "Image has neither url nor data".into(),
            details: ProviderDetails {
                provider: Some("anthropic".to_string()),
                ..ProviderDetails::default()
            },
        })
    }
}

fn translate_document(document: &crate::types::content::DocumentData) -> SdkResult<Value> {
    document.validate()?;
    if let Some(url) = &document.url {
        Ok(json!({
            "type": "document",
            "source": {"type": "url", "url": url}
        }))
    } else if let Some(data) = &document.data {
        use base64::Engine;
        let media_type = document.media_type.as_deref().unwrap_or("application/pdf");
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        Ok(json!({
            "type": "document",
            "source": {"type": "base64", "media_type": media_type, "data": encoded}
        }))
    } else {
        // validate() above ensures this branch is unreachable
        Err(SdkError::InvalidRequest {
            message: "DocumentData: neither url nor data is set".into(),
            details: ProviderDetails {
                provider: Some("anthropic".to_string()),
                ..ProviderDetails::default()
            },
        })
    }
}

fn translate_tool_result(tool_result: &crate::types::content::ToolResultData) -> Value {
    let content_value = if let Some(s) = tool_result.content.as_str() {
        Value::String(s.to_string())
    } else {
        Value::String(tool_result.content.to_string())
    };
    let mut block = json!({
        "type": "tool_result",
        "tool_use_id": tool_result.tool_call_id,
        "content": content_value
    });
    if tool_result.is_error {
        block["is_error"] = Value::Bool(true);
    }
    block
}

/// Merge consecutive messages with the same role to satisfy Anthropic's
/// strict alternation requirement.
fn merge_consecutive_messages(messages: Vec<Value>) -> Vec<Value> {
    let mut merged: Vec<Value> = Vec::new();

    for msg in messages {
        let role = msg.get("role").and_then(Value::as_str).unwrap_or_default();
        let content = msg
            .get("content")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()));

        if let Some(last) = merged.last_mut()
            && last.get("role").and_then(Value::as_str) == Some(role)
        {
            // Same role: merge content arrays
            if let Some(existing_content) = last.get_mut("content").and_then(Value::as_array_mut)
                && let Some(new_content) = content.as_array()
            {
                existing_content.extend(new_content.iter().cloned());
            }
        } else {
            merged.push(msg);
        }
    }

    merged
}

fn translate_tool_definition(tool: &ToolDefinition) -> SdkResult<Value> {
    tool.validate()?;

    Ok(json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters
    }))
}

enum ToolChoiceAction {
    SetChoice(Value),
    OmitTools,
}

fn translate_tool_choice(tool_choice: &ToolChoice) -> ToolChoiceAction {
    match tool_choice {
        ToolChoice::Auto => ToolChoiceAction::SetChoice(json!({"type": "auto"})),
        ToolChoice::None => ToolChoiceAction::OmitTools,
        ToolChoice::Required => ToolChoiceAction::SetChoice(json!({"type": "any"})),
        ToolChoice::Tool(name) => {
            ToolChoiceAction::SetChoice(json!({"type": "tool", "name": name}))
        }
    }
}

fn apply_provider_options(
    options: &Value,
    body: &mut Map<String, Value>,
    headers: &mut HeaderMap,
    auto_cache: &mut bool,
) -> SdkResult<()> {
    let Some(options_obj) = options.as_object() else {
        return Err(SdkError::InvalidRequest {
            message: "provider_options.anthropic must be an object".to_string(),
            details: ProviderDetails {
                provider: Some("anthropic".to_string()),
                ..ProviderDetails::default()
            },
        });
    };

    // Handle beta headers
    if let Some(beta_value) = options_obj
        .get("beta_headers")
        .or_else(|| options_obj.get("beta_features"))
    {
        if let Some(arr) = beta_value.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    add_beta_header(headers, s);
                }
            }
        } else if let Some(s) = beta_value.as_str() {
            for part in s.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    add_beta_header(headers, trimmed);
                }
            }
        }
    }

    // Handle auto_cache option
    if let Some(cache_value) = options_obj.get("auto_cache")
        && let Some(b) = cache_value.as_bool()
    {
        *auto_cache = b;
    }

    // Pass through remaining options to the request body
    for (key, value) in options_obj {
        if matches!(
            key.as_str(),
            "beta_headers" | "beta_features" | "auto_cache"
        ) {
            continue;
        }
        body.insert(key.clone(), value.clone());
    }

    Ok(())
}

/// Add a beta feature to the `anthropic-beta` header, avoiding duplicates.
fn add_beta_header(headers: &mut HeaderMap, feature: &str) {
    let header_name = HeaderName::from_static("anthropic-beta");

    let existing = headers
        .get(&header_name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    // Check for duplicate
    let features: Vec<&str> = if existing.is_empty() {
        Vec::new()
    } else {
        existing.split(',').map(str::trim).collect()
    };

    if features.contains(&feature) {
        return;
    }

    let new_value = if existing.is_empty() {
        feature.to_string()
    } else {
        format!("{existing},{feature}")
    };

    if let Ok(value) = HeaderValue::from_str(&new_value) {
        headers.insert(header_name, value);
    }
}

/// Inject `cache_control` markers on the last system block, last tool definition,
/// and the conversation prefix (the second-to-last message, which represents the
/// stable portion of a multi-turn conversation).
///
/// Per spec Section 2.10 / 8.6: the adapter must inject breakpoints on the system
/// prompt, tool definitions, and conversation prefix for agentic workloads.
fn inject_cache_control(body: &mut Map<String, Value>) {
    // Add cache_control to the last system block
    if let Some(system) = body.get_mut("system").and_then(Value::as_array_mut)
        && let Some(last) = system.last_mut()
        && let Some(obj) = last.as_object_mut()
    {
        obj.insert("cache_control".to_string(), json!({"type": "ephemeral"}));
    }

    // Add cache_control to the last tool definition
    if let Some(tools) = body.get_mut("tools").and_then(Value::as_array_mut)
        && let Some(last) = tools.last_mut()
        && let Some(obj) = last.as_object_mut()
    {
        obj.insert("cache_control".to_string(), json!({"type": "ephemeral"}));
    }

    // Add cache_control to the conversation prefix: the second-to-last message.
    // In multi-turn agentic workloads, the final message is the new user turn
    // while everything before it is the stable prefix. Marking the boundary lets
    // Anthropic cache all prior turns.
    if let Some(messages) = body.get_mut("messages").and_then(Value::as_array_mut)
        && messages.len() >= 2
    {
        let prefix_idx = messages.len() - 2;
        if let Some(msg) = messages.get_mut(prefix_idx)
            && let Some(msg_obj) = msg.as_object_mut()
        {
            // Inject on the last content block within the message
            if let Some(content) = msg_obj.get_mut("content").and_then(Value::as_array_mut)
                && let Some(last_block) = content.last_mut()
                && let Some(block_obj) = last_block.as_object_mut()
            {
                block_obj.insert("cache_control".to_string(), json!({"type": "ephemeral"}));
            }
        }
    }
}
