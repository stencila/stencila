use base64::Engine;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, Value, json};

use crate::error::{ProviderDetails, SdkError, SdkResult};
use crate::providers::common::image::read_local_image_from_url;
use crate::types::content::ImageData;
use crate::types::response_format::{ResponseFormat, ResponseFormatType};
use crate::types::tool::ToolChoice;

/// ID prefixes for non-chat OpenAI models (embeddings, image gen, TTS, etc.).
const EXCLUDED_OPENAI_PREFIXES: &[&str] = &[
    "text-embedding-",
    "dall-e-",
    "gpt-image-",
    "tts-",
    "whisper-",
    "sora-",
    "davinci-",
    "babbage-",
    "codex-",
    "omni-moderation-",
    "chatgpt-image-",
    "computer-use-",
];

/// Substrings that indicate non-chat OpenAI model variants.
const EXCLUDED_OPENAI_SUBSTRINGS: &[&str] = &[
    "-tts",
    "-realtime",
    "-audio-",
    "-transcribe",
    "-search-",
    "-deep-research",
];

/// Returns `true` if the given OpenAI model ID should be excluded from the
/// catalog because it is not a chat/completions model.
pub(crate) fn is_excluded_openai_model(id: &str) -> bool {
    EXCLUDED_OPENAI_PREFIXES
        .iter()
        .any(|prefix| id.starts_with(prefix))
        || EXCLUDED_OPENAI_SUBSTRINGS
            .iter()
            .any(|sub| id.contains(sub))
}

/// Translate a unified `ToolChoice` into the OpenAI JSON shape.
///
/// This is identical for both the Responses API and the Chat Completions API.
pub(crate) fn translate_tool_choice(tool_choice: &ToolChoice) -> Value {
    match tool_choice {
        ToolChoice::Auto => Value::String("auto".to_string()),
        ToolChoice::None => Value::String("none".to_string()),
        ToolChoice::Required => Value::String("required".to_string()),
        ToolChoice::Tool(name) => json!({
            "type": "function",
            "function": {"name": name}
        }),
    }
}

/// Translate a unified `ResponseFormat` into the OpenAI JSON shape.
///
/// This is identical for both the Responses API and the Chat Completions API.
pub(crate) fn translate_response_format(format: &ResponseFormat) -> Value {
    match format.format_type {
        ResponseFormatType::Text => json!({"type": "text"}),
        ResponseFormatType::Json => json!({"type": "json_object"}),
        ResponseFormatType::JsonSchema => json!({
            "type": "json_schema",
            "json_schema": {
                "name": "response",
                "schema": format.json_schema.clone().unwrap_or(Value::Null),
                "strict": format.strict
            }
        }),
    }
}

/// Convert an `ImageData` to an OpenAI-compatible URL (either a plain URL or a
/// `data:` URI with base64-encoded data).
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` when image data is missing.
pub(crate) fn image_to_openai_url(image: &ImageData, provider: &str) -> SdkResult<String> {
    if let Some(url) = &image.url {
        if let Some((data, media_type)) =
            read_local_image_from_url(url, image.media_type.as_deref(), provider)?
        {
            let encoded = base64::engine::general_purpose::STANDARD.encode(data);
            return Ok(format!("data:{media_type};base64,{encoded}"));
        }
        return Ok(url.clone());
    }

    let data = image
        .data
        .as_ref()
        .ok_or_else(|| SdkError::InvalidRequest {
            message: "image data missing".to_string(),
            details: ProviderDetails {
                provider: Some(provider.to_string()),
                ..ProviderDetails::default()
            },
        })?;

    let media_type = image.effective_media_type().unwrap_or("image/png");
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    Ok(format!("data:{media_type};base64,{encoded}"))
}

/// Serialize a `serde_json::Value` to a JSON string. If the value is already a
/// `Value::String`, returns it directly without re-quoting.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` on serialization failure.
pub(crate) fn value_to_json_string(value: &Value, provider: &str) -> SdkResult<String> {
    if let Some(s) = value.as_str() {
        Ok(s.to_string())
    } else {
        serde_json::to_string(value).map_err(|e| SdkError::InvalidRequest {
            message: format!("unable to serialize JSON value: {e}"),
            details: ProviderDetails {
                provider: Some(provider.to_string()),
                ..ProviderDetails::default()
            },
        })
    }
}

/// Parse custom headers from a provider options object and insert them into a
/// `HeaderMap`.
///
/// Looks for `custom_headers` or `headers` keys in `options_obj`.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` on invalid header names/values.
pub(crate) fn parse_custom_headers(
    options_obj: &Map<String, Value>,
    headers: &mut HeaderMap,
    provider: &str,
) -> SdkResult<()> {
    let Some(custom_headers) = options_obj
        .get("custom_headers")
        .or_else(|| options_obj.get("headers"))
    else {
        return Ok(());
    };

    let Some(custom_headers_obj) = custom_headers.as_object() else {
        return Err(SdkError::InvalidRequest {
            message: format!("provider_options.{provider}.custom_headers must be an object"),
            details: ProviderDetails {
                provider: Some(provider.to_string()),
                ..ProviderDetails::default()
            },
        });
    };

    for (name, value) in custom_headers_obj {
        let value = value.as_str().ok_or_else(|| SdkError::InvalidRequest {
            message: format!("custom header value must be string for key '{name}'"),
            details: ProviderDetails {
                provider: Some(provider.to_string()),
                ..ProviderDetails::default()
            },
        })?;

        let header_name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|e| SdkError::InvalidRequest {
                message: format!("invalid custom header name '{name}': {e}"),
                details: ProviderDetails {
                    provider: Some(provider.to_string()),
                    ..ProviderDetails::default()
                },
            })?;

        let header_value = HeaderValue::from_str(value).map_err(|e| SdkError::InvalidRequest {
            message: format!("invalid custom header value for '{name}': {e}"),
            details: ProviderDetails {
                provider: Some(provider.to_string()),
                ..ProviderDetails::default()
            },
        })?;

        headers.insert(header_name, header_value);
    }

    Ok(())
}
