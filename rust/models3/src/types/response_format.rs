use serde::{Deserialize, Serialize};

/// Controls the format of the model's response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// The response format type: "text", "json", or `json_schema`.
    #[serde(rename = "type")]
    pub format_type: ResponseFormatType,
    /// JSON Schema to enforce (required when `format_type` is `JsonSchema`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
    /// When true, provider enforces the schema strictly.
    #[serde(default)]
    pub strict: bool,
}

/// The type of response format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    /// Plain text output.
    Text,
    /// JSON output (no specific schema).
    Json,
    /// JSON output conforming to a specific schema.
    JsonSchema,
}
