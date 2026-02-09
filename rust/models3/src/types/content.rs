use serde::{Deserialize, Serialize};

use crate::error::{ProviderDetails, SdkError, SdkResult};

/// A single part of a message's multimodal content.
///
/// Uses a Rust enum (tagged union) with a variant per content kind.
/// The `Extension` variant preserves unknown kinds for forward compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentPart {
    /// Plain text content.
    Text { text: String },
    /// Image content as URL or inline bytes.
    Image { image: ImageData },
    /// Audio content as URL or inline bytes.
    Audio { audio: AudioData },
    /// Document content (PDF, etc.) as URL or inline bytes.
    Document { document: DocumentData },
    /// A tool call initiated by the model.
    ToolCall { tool_call: ToolCallData },
    /// The result of executing a tool call.
    ToolResult { tool_result: ToolResultData },
    /// Model reasoning/thinking content.
    Thinking { thinking: ThinkingData },
    /// Redacted reasoning (Anthropic). Must be round-tripped verbatim.
    RedactedThinking { thinking: ThinkingData },
    /// Unknown content kind for forward compatibility.
    #[serde(untagged)]
    Extension(serde_json::Value),
}

impl ContentPart {
    /// Create a text content part.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create an image content part from a URL.
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::Image {
            image: ImageData {
                url: Some(url.into()),
                data: None,
                media_type: None,
                detail: None,
            },
        }
    }

    /// Create an image content part from raw bytes.
    pub fn image_data(data: Vec<u8>, media_type: impl Into<String>) -> Self {
        Self::Image {
            image: ImageData {
                url: None,
                data: Some(data),
                media_type: Some(media_type.into()),
                detail: None,
            },
        }
    }

    /// Create a tool call content part.
    pub fn tool_call(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> Self {
        Self::ToolCall {
            tool_call: ToolCallData {
                id: id.into(),
                name: name.into(),
                arguments,
                call_type: "function".to_string(),
            },
        }
    }

    /// Create a tool result content part.
    pub fn tool_result(
        tool_call_id: impl Into<String>,
        content: serde_json::Value,
        is_error: bool,
    ) -> Self {
        Self::ToolResult {
            tool_result: ToolResultData {
                tool_call_id: tool_call_id.into(),
                content,
                is_error,
                image_data: None,
                image_media_type: None,
            },
        }
    }

    /// Extract the text from this content part, if it is a text part.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Known content kind names used by the tagged enum variants.
    const KNOWN_KINDS: &'static [&'static str] = &[
        "text",
        "image",
        "audio",
        "document",
        "tool_call",
        "tool_result",
        "thinking",
        "redacted_thinking",
    ];

    /// Validate that this content part is structurally sound.
    ///
    /// In particular, catches the case where a malformed payload with a
    /// known `kind` (e.g. `{"kind":"text"}` without a `text` field) was
    /// silently deserialized as `Extension` by serde's untagged fallback.
    /// Provider adapters should call this after deserialization.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::InvalidRequest` if an `Extension` value has a
    /// `kind` matching a known variant name.
    pub fn validate(&self) -> SdkResult<()> {
        if let Self::Extension(value) = self
            && let Some(kind) = value.get("kind").and_then(|v| v.as_str())
            && Self::KNOWN_KINDS.contains(&kind)
        {
            return Err(SdkError::InvalidRequest {
                message: format!(
                    "ContentPart has known kind \"{kind}\" but failed structural \
                     validation — deserialized as Extension fallback"
                ),
                details: ProviderDetails::default(),
            });
        }
        Ok(())
    }
}

/// Image data: exactly one of `url` or `data` must be provided.
///
/// When `data` is set without `media_type`, the media type defaults
/// to `"image/png"` per spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_bytes",
        deserialize_with = "deserialize_opt_bytes",
        default
    )]
    pub data: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// OpenAI-specific detail level: "auto", "low", or "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ImageData {
    /// Validate that exactly one of `url` or `data` is set.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::InvalidRequest` if both or neither are set.
    pub fn validate(&self) -> SdkResult<()> {
        match (&self.url, &self.data) {
            (Some(_), Some(_)) => Err(SdkError::InvalidRequest {
                message: "ImageData: both url and data are set; exactly one required".into(),
                details: ProviderDetails::default(),
            }),
            (None, None) => Err(SdkError::InvalidRequest {
                message: "ImageData: neither url nor data is set; exactly one required".into(),
                details: ProviderDetails::default(),
            }),
            _ => Ok(()),
        }
    }

    /// Return the effective media type, applying the spec default
    /// (`"image/png"`) when `data` is present without an explicit type.
    #[must_use]
    pub fn effective_media_type(&self) -> Option<&str> {
        match (&self.media_type, &self.data) {
            (Some(mt), _) => Some(mt.as_str()),
            (None, Some(_)) => Some("image/png"),
            (None, None) => None,
        }
    }
}

/// Audio data as URL or raw bytes with media type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_bytes",
        deserialize_with = "deserialize_opt_bytes",
        default
    )]
    pub data: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

/// Document data (PDF, etc.) as URL or raw bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_bytes",
        deserialize_with = "deserialize_opt_bytes",
        default
    )]
    pub data: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

/// Data for a tool call initiated by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallData {
    /// Unique identifier, provider-assigned.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Parsed JSON arguments (or raw string wrapped in `Value::String`).
    pub arguments: serde_json::Value,
    /// Call type: "function" (default) or "custom".
    #[serde(
        rename = "type",
        default = "ToolCallData::default_type",
        skip_serializing_if = "ToolCallData::is_default_type"
    )]
    pub call_type: String,
}

impl ToolCallData {
    fn default_type() -> String {
        "function".to_string()
    }

    fn is_default_type(s: &str) -> bool {
        s == "function"
    }
}

/// Data for the result of executing a tool call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResultData {
    /// The `ToolCallData.id` this result answers.
    pub tool_call_id: String,
    /// Tool output (string or structured).
    pub content: serde_json::Value,
    /// Whether tool execution failed.
    #[serde(default)]
    pub is_error: bool,
    /// Optional image result bytes.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_bytes",
        deserialize_with = "deserialize_opt_bytes",
        default
    )]
    pub image_data: Option<Vec<u8>>,
    /// MIME type for the image result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_media_type: Option<String>,
}

/// Thinking/reasoning content from the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThinkingData {
    /// The thinking/reasoning text content.
    pub text: String,
    /// Provider-specific signature for round-tripping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// True if this is redacted thinking with opaque content.
    #[serde(default)]
    pub redacted: bool,
}

// --- Base64 serde helpers for Option<Vec<u8>> ---

// The `&Option<T>` signature is required by serde's `serialize_with` attribute.
#[allow(clippy::ref_option)]
fn serialize_opt_bytes<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use base64::Engine;
    match value {
        Some(bytes) => {
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            serializer.serialize_some(&encoded)
        }
        None => serializer.serialize_none(),
    }
}

fn deserialize_opt_bytes<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use base64::Engine;
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&s)
                .map_err(serde::de::Error::custom)?;
            Ok(Some(bytes))
        }
        None => Ok(None),
    }
}
