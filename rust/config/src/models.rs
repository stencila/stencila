//! Configuration for model provider selection.

use eyre::{Result, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Known provider identifiers accepted in `models.providers`.
pub const KNOWN_MODEL_PROVIDERS: &[&str] = &[
    "anthropic",
    "deepseek",
    "gemini",
    "mistral",
    "ollama",
    "openai",
];

/// Models configuration.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelsConfig {
    /// Ordered provider list used for model provider selection.
    ///
    /// When set, this list controls which providers are considered and in which order.
    /// Example:
    ///
    /// ```toml
    /// [models]
    /// providers = ["anthropic", "openai"]
    /// ```
    pub providers: Option<Vec<String>>,

    /// Ollama-specific configuration for local model usage.
    ///
    /// Example:
    ///
    /// ```toml
    /// [models.ollama]
    /// base_url = "http://my-gpu-server:11434/v1"
    /// default_model = "llama3.1:8b"
    /// ```
    pub ollama: Option<OllamaConfig>,
}

/// Ollama-specific configuration.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OllamaConfig {
    /// Base URL for the Ollama API.
    ///
    /// Defaults to `http://localhost:11434/v1`.
    /// Overrides the `OLLAMA_BASE_URL` and `OLLAMA_HOST` environment variables.
    pub base_url: Option<String>,

    /// Default model to use when no model is specified.
    ///
    /// Should match a model that has been pulled locally (e.g. `"llama3.1:8b"`).
    pub default_model: Option<String>,

    /// Whether to auto-detect a running Ollama instance at `localhost:11434`.
    ///
    /// Defaults to `true`. Set to `false` to disable auto-detection and only
    /// register Ollama when explicitly configured via environment variables or
    /// `base_url`.
    pub auto_detect: Option<bool>,
}

impl ModelsConfig {
    /// Validate `models` configuration.
    ///
    /// Returns an error for unknown provider names or duplicates.
    pub fn validate(&self) -> Result<()> {
        let Some(providers) = &self.providers else {
            return Ok(());
        };

        let mut seen = std::collections::HashSet::new();
        for provider in providers {
            if !KNOWN_MODEL_PROVIDERS.contains(&provider.as_str()) {
                return Err(eyre!(
                    "Unknown model provider '{provider}' in [models].providers. Known providers: {}",
                    KNOWN_MODEL_PROVIDERS.join(", ")
                ));
            }

            if !seen.insert(provider) {
                return Err(eyre!(
                    "Duplicate model provider '{provider}' in [models].providers"
                ));
            }
        }

        Ok(())
    }
}
