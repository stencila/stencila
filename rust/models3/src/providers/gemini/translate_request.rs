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

                let mut part_obj = json!({
                    "functionCall": {
                        "name": tool_call.name,
                        "args": args
                    }
                });
                if let Some(sig) = &tool_call.thought_signature {
                    part_obj["thoughtSignature"] = Value::String(sig.clone());
                }
                result.push(part_obj);
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
            ContentPart::Thinking { thinking } => {
                let mut part_obj = json!({
                    "text": thinking.text,
                    "thought": true
                });
                if let Some(sig) = &thinking.signature {
                    part_obj["thoughtSignature"] = Value::String(sig.clone());
                }
                result.push(part_obj);
            }
            // Redacted thinking is Anthropic-specific opaque content — skip when
            // translating to Gemini to avoid leaking provider-internal payloads.
            ContentPart::RedactedThinking { .. } | ContentPart::Extension(_) => {}
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

    let mut params = tool.parameters.clone();
    strip_unsupported_schema_fields(&mut params);
    decl.insert("parameters".to_string(), params);

    Ok(Value::Object(decl))
}

#[allow(clippy::doc_markdown)]
/// Sanitize a JSON Schema value for the Gemini API.
///
/// Gemini uses a subset of the OpenAPI 3.0 Schema Object and rejects
/// standard JSON Schema keywords it does not recognise (`$schema`,
/// `$ref`, `$defs`/`definitions`, `additionalProperties`, `title`,
/// `default`). This function:
///
/// 1. Collects `$defs` / `definitions` from the root so that `$ref`
///    pointers can be resolved.
/// 2. Recursively inlines every `$ref` with the referenced definition.
/// 3. Strips keywords that Gemini does not accept.
fn strip_unsupported_schema_fields(value: &mut Value) {
    // Collect definitions from the root before mutating.
    let defs = collect_defs(value);

    // Inline $ref pointers and strip unsupported keywords.
    strip_recursive(value, &defs);
}

/// Collect the `$defs` (or `definitions`) map from the root schema value.
fn collect_defs(root: &Value) -> Map<String, Value> {
    if let Some(obj) = root.as_object()
        && let Some(Value::Object(d)) = obj.get("$defs").or_else(|| obj.get("definitions"))
    {
        return d.clone();
    }
    Map::new()
}

/// Fields that Gemini does not support in function-declaration schemas.
const UNSUPPORTED_KEYS: &[&str] = &[
    "$schema",
    "$defs",
    "definitions",
    "additionalProperties",
    "title",
    "default",
];

fn strip_recursive(value: &mut Value, defs: &Map<String, Value>) {
    match value {
        Value::Object(obj) => {
            // Resolve `$ref` — whether standalone or mixed with sibling keys
            // like `description` or `default`. The resolved definition is
            // merged in and the `$ref` key removed.
            if let Some(ref_str) = obj.get("$ref").and_then(|v| v.as_str()).map(String::from)
                && let Some(Value::Object(resolved)) = resolve_ref(&ref_str, defs)
            {
                obj.remove("$ref");
                // Merge resolved fields; existing sibling keys (e.g.
                // `description`) take precedence.
                for (k, v) in resolved {
                    obj.entry(k).or_insert(v);
                }
            }

            // Convert `"type": ["string", "null"]` → `"type": "string"` +
            // `"nullable": true`.  Gemini does not accept type-arrays.
            if let Some(type_val) = obj.get("type")
                && type_val.is_array()
            {
                let arr = type_val.as_array().expect("checked is_array");
                let has_null = arr.iter().any(|v| v.as_str() == Some("null"));
                let non_null: Vec<&Value> =
                    arr.iter().filter(|v| v.as_str() != Some("null")).collect();
                if non_null.len() == 1 {
                    obj.insert("type".to_string(), non_null[0].clone());
                }
                if has_null {
                    obj.insert("nullable".to_string(), Value::Bool(true));
                }
            }

            // Remove unsupported keys.
            for key in UNSUPPORTED_KEYS {
                obj.remove(*key);
            }

            // Recurse into remaining values.
            for v in obj.values_mut() {
                strip_recursive(v, defs);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_recursive(v, defs);
            }
        }
        _ => {}
    }
}

/// Resolve a `$ref` string like `#/$defs/Foo` or `#/definitions/Foo`
/// against the collected definitions map.
fn resolve_ref(ref_str: &str, defs: &Map<String, Value>) -> Option<Value> {
    let name = ref_str
        .strip_prefix("#/$defs/")
        .or_else(|| ref_str.strip_prefix("#/definitions/"))?;
    defs.get(name).cloned()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strip_removes_schema_keyword() {
        let mut schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });
        strip_unsupported_schema_fields(&mut schema);
        assert!(schema.get("$schema").is_none());
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn strip_removes_additional_properties() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "x": { "type": "integer", "additionalProperties": false }
            },
            "additionalProperties": false
        });
        strip_unsupported_schema_fields(&mut schema);
        assert!(schema.get("additionalProperties").is_none());
        assert!(
            schema["properties"]["x"]
                .get("additionalProperties")
                .is_none()
        );
    }

    #[test]
    fn strip_inlines_ref_from_defs() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "item": { "$ref": "#/$defs/Item" }
            },
            "$defs": {
                "Item": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string" }
                    }
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        // $defs should be removed
        assert!(schema.get("$defs").is_none());

        // $ref should be resolved inline
        let item = &schema["properties"]["item"];
        assert_eq!(item["type"], "object");
        assert_eq!(item["properties"]["label"]["type"], "string");
    }

    #[test]
    fn strip_inlines_ref_from_definitions() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "addr": { "$ref": "#/definitions/Address" }
            },
            "definitions": {
                "Address": {
                    "type": "object",
                    "properties": {
                        "city": { "type": "string" }
                    }
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        assert!(schema.get("definitions").is_none());
        assert_eq!(schema["properties"]["addr"]["type"], "object");
    }

    #[test]
    fn strip_handles_nested_refs() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": { "$ref": "#/$defs/Option" }
                }
            },
            "$defs": {
                "Option": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        let option_schema = &schema["properties"]["items"]["items"];
        assert_eq!(option_schema["type"], "object");
        assert!(option_schema.get("additionalProperties").is_none());
    }

    #[test]
    fn strip_removes_title_and_default() {
        let mut schema = json!({
            "type": "object",
            "title": "MySchema",
            "properties": {
                "name": {
                    "type": "string",
                    "default": "unnamed",
                    "title": "Name"
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        assert!(schema.get("title").is_none());
        assert!(schema["properties"]["name"].get("default").is_none());
        assert!(schema["properties"]["name"].get("title").is_none());
    }

    #[test]
    fn strip_converts_nullable_type_array() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": ["string", "null"]
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        let name = &schema["properties"]["name"];
        assert_eq!(name["type"], "string");
        assert_eq!(name["nullable"], true);
    }

    #[test]
    fn strip_resolves_ref_with_sibling_keys() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "kind": {
                    "description": "The kind of thing.",
                    "$ref": "#/$defs/Kind",
                    "default": "a"
                }
            },
            "$defs": {
                "Kind": {
                    "type": "string",
                    "enum": ["a", "b", "c"]
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        let kind = &schema["properties"]["kind"];
        // $ref resolved: type and enum inlined
        assert_eq!(kind["type"], "string");
        assert!(kind.get("$ref").is_none());
        // sibling description preserved
        assert_eq!(kind["description"], "The kind of thing.");
        // default stripped (unsupported by Gemini)
        assert!(kind.get("default").is_none());
    }

    #[test]
    fn strip_handles_actual_interview_spec_schema() {
        // Mirrors the real schemars output for InterviewSpec
        let mut schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "InterviewSpec",
            "type": "object",
            "properties": {
                "preamble": {
                    "description": "Markdown preamble.",
                    "type": ["string", "null"]
                },
                "questions": {
                    "description": "Questions.",
                    "type": "array",
                    "items": { "$ref": "#/$defs/QuestionSpec" }
                }
            },
            "required": ["questions"],
            "$defs": {
                "QuestionSpec": {
                    "description": "A question.",
                    "type": "object",
                    "properties": {
                        "question": { "type": "string" },
                        "header": { "type": ["string", "null"] },
                        "type": {
                            "description": "Question type.",
                            "$ref": "#/$defs/QuestionTypeSpec",
                            "default": "freeform"
                        },
                        "options": {
                            "type": "array",
                            "items": { "$ref": "#/$defs/OptionSpec" }
                        },
                        "default": { "type": ["string", "null"] },
                        "store": { "type": ["string", "null"] },
                        "finish_if": { "type": ["string", "null"] },
                        "show_if": { "type": ["string", "null"] }
                    },
                    "required": ["question"]
                },
                "QuestionTypeSpec": {
                    "type": "string",
                    "enum": ["yes-no", "confirm", "single-select", "multi-select", "freeform"]
                },
                "OptionSpec": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string" },
                        "description": { "type": ["string", "null"] }
                    },
                    "required": ["label"]
                }
            }
        });
        strip_unsupported_schema_fields(&mut schema);

        // Root-level unsupported keys removed
        assert!(schema.get("$schema").is_none());
        assert!(schema.get("$defs").is_none());
        assert!(schema.get("title").is_none());

        // Nullable preamble converted
        let preamble = &schema["properties"]["preamble"];
        assert_eq!(preamble["type"], "string");
        assert_eq!(preamble["nullable"], true);

        // Questions items resolved from $ref
        let q = &schema["properties"]["questions"]["items"];
        assert_eq!(q["type"], "object");
        assert!(q.get("$ref").is_none());

        // Nullable fields inside question converted
        assert_eq!(q["properties"]["header"]["type"], "string");
        assert_eq!(q["properties"]["header"]["nullable"], true);
        assert_eq!(q["properties"]["store"]["type"], "string");
        assert_eq!(q["properties"]["finish_if"]["type"], "string");
        assert_eq!(q["properties"]["show_if"]["type"], "string");

        // type field: $ref resolved to enum, default stripped
        let type_prop = &q["properties"]["type"];
        assert_eq!(type_prop["type"], "string");
        assert!(type_prop.get("$ref").is_none());
        assert!(type_prop.get("default").is_none());
        assert!(type_prop["enum"].is_array());

        // Nested OptionSpec resolved
        let opt = &q["properties"]["options"]["items"];
        assert_eq!(opt["type"], "object");
        assert!(opt.get("$ref").is_none());
        assert_eq!(opt["properties"]["description"]["type"], "string");
        assert_eq!(opt["properties"]["description"]["nullable"], true);
    }
}
