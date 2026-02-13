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
