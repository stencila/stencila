use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::message::Message;
use super::response_format::ResponseFormat;
use super::timeout::Timeout;
use super::tool::{ToolChoice, ToolDefinition};

/// A unified request to an LLM provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// The model identifier (provider's native model ID).
    pub model: String,
    /// The conversation messages.
    pub messages: Vec<Message>,
    /// Which provider to route to (uses default if omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Tool definitions available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// Controls how the model uses tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Controls the response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Reasoning effort level: "none", "low", "medium", "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Arbitrary key-value metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Per-provider escape hatch for provider-specific parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<HashMap<String, serde_json::Value>>,
    /// Timeout configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Timeout>,
}

impl Request {
    /// Create a minimal request with a model and messages.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            provider: None,
            tools: None,
            tool_choice: None,
            response_format: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop_sequences: None,
            reasoning_effort: None,
            metadata: None,
            provider_options: None,
            timeout: None,
        }
    }

    /// Extract the provider-specific options for a given provider name.
    #[must_use]
    pub fn provider_options_for(&self, provider: &str) -> Option<&serde_json::Value> {
        self.provider_options
            .as_ref()
            .and_then(|opts| opts.get(provider))
    }
}
