use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::error::{SdkError, SdkResult};

/// Static model catalog loaded from embedded JSON.
///
/// Stores a `Result` to avoid panicking if the embedded JSON is malformed.
/// In practice this cannot fail since the JSON is embedded at compile time,
/// but propagating an error is more consistent with the crate's guidelines.
static CATALOG: LazyLock<Result<Vec<ModelInfo>, String>> = LazyLock::new(|| {
    let json = include_str!("catalog/models.json");
    serde_json::from_str(json).map_err(|e| e.to_string())
});

/// Access the parsed catalog, mapping a parse failure to `SdkError::Configuration`.
fn catalog() -> SdkResult<&'static Vec<ModelInfo>> {
    CATALOG.as_ref().map_err(|msg| SdkError::Configuration {
        message: format!("catalog/models.json parse error: {msg}"),
    })
}

/// Information about a known model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model's API identifier (e.g. `claude-opus-4-6`).
    pub id: String,
    /// Which provider serves this model (e.g. `anthropic`).
    pub provider: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Maximum total tokens (input + output).
    pub context_window: u64,
    /// Maximum output tokens, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output: Option<u64>,
    /// Whether the model supports tool calling.
    #[serde(default)]
    pub supports_tools: bool,
    /// Whether the model accepts image inputs.
    #[serde(default)]
    pub supports_vision: bool,
    /// Whether the model produces reasoning tokens.
    #[serde(default)]
    pub supports_reasoning: bool,
    /// Cost per 1M input tokens (USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cost_per_million: Option<f64>,
    /// Cost per 1M output tokens (USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_cost_per_million: Option<f64>,
    /// Shorthand aliases (e.g. `["sonnet", "claude-sonnet"]`).
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Look up a model by its ID or alias. Returns `None` for unknown models.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the embedded catalog JSON is malformed.
pub fn get_model_info(model_id: &str) -> SdkResult<Option<&'static ModelInfo>> {
    Ok(catalog()?
        .iter()
        .find(|m| m.id == model_id || m.aliases.iter().any(|alias| alias == model_id)))
}

/// List all known models, optionally filtered by provider.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the embedded catalog JSON is malformed.
pub fn list_models(provider: Option<&str>) -> SdkResult<Vec<&'static ModelInfo>> {
    Ok(match provider {
        Some(p) => catalog()?.iter().filter(|m| m.provider == p).collect(),
        None => catalog()?.iter().collect(),
    })
}

/// Return the first (newest/best) model for a provider, optionally
/// filtered by capability.
///
/// The catalog is ordered with the newest/best models first per provider,
/// so the first match is the "latest".
///
/// Known capabilities: `"tools"`, `"vision"`, `"reasoning"`.
/// Returns `None` for unknown capability values (surfaces typos).
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the embedded catalog JSON is malformed.
pub fn get_latest_model(
    provider: &str,
    capability: Option<&str>,
) -> SdkResult<Option<&'static ModelInfo>> {
    Ok(catalog()?.iter().find(|m| {
        m.provider == provider
            && match capability {
                None => true,
                Some("tools") => m.supports_tools,
                Some("vision") => m.supports_vision,
                Some("reasoning") => m.supports_reasoning,
                Some(_) => false, // unknown capability → no match
            }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_loads() -> Result<(), Box<dyn std::error::Error>> {
        let models = list_models(None)?;
        assert!(!models.is_empty(), "catalog should contain models");
        Ok(())
    }

    #[test]
    fn lookup_by_id() -> Result<(), Box<dyn std::error::Error>> {
        let all = list_models(None)?;
        let first = all.first().ok_or("catalog is empty")?;
        let info = get_model_info(&first.id)?.ok_or("lookup by id failed")?;
        assert_eq!(info.id, first.id);
        Ok(())
    }

    #[test]
    fn lookup_by_alias() -> Result<(), Box<dyn std::error::Error>> {
        let all = list_models(None)?;
        let with_alias = all
            .iter()
            .find(|m| !m.aliases.is_empty())
            .ok_or("no models with aliases")?;
        let info = get_model_info(&with_alias.aliases[0])?.ok_or("alias lookup failed")?;
        assert_eq!(info.id, with_alias.id);
        Ok(())
    }

    #[test]
    fn unknown_model_returns_none() -> Result<(), Box<dyn std::error::Error>> {
        assert!(get_model_info("nonexistent-model-xyz")?.is_none());
        Ok(())
    }

    #[test]
    fn list_by_provider() -> Result<(), Box<dyn std::error::Error>> {
        let all = list_models(None)?;
        let provider = &all.first().ok_or("catalog is empty")?.provider;
        let filtered = list_models(Some(provider))?;
        assert!(!filtered.is_empty());
        for m in &filtered {
            assert_eq!(&m.provider, provider);
        }
        Ok(())
    }

    #[test]
    fn latest_with_capability() -> Result<(), Box<dyn std::error::Error>> {
        let all = list_models(None)?;
        let reasoning = all
            .iter()
            .find(|m| m.supports_reasoning)
            .ok_or("no reasoning models")?;
        let m = get_latest_model(&reasoning.provider, Some("reasoning"))?
            .ok_or("latest reasoning lookup failed")?;
        assert!(m.supports_reasoning);
        Ok(())
    }

    #[test]
    fn latest_unknown_provider() -> Result<(), Box<dyn std::error::Error>> {
        assert!(get_latest_model("nonexistent", None)?.is_none());
        Ok(())
    }

    #[test]
    fn latest_unknown_capability_returns_none() -> Result<(), Box<dyn std::error::Error>> {
        let all = list_models(None)?;
        let provider = &all.first().ok_or("catalog is empty")?.provider;
        // Typo "vison" should return None, not silently match
        assert!(get_latest_model(provider, Some("vison"))?.is_none());
        Ok(())
    }
}
