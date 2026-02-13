use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use stencila_auth::{AuthOverrides, claude_code, codex_cli};

use crate::client::Client;
use crate::error::{SdkError, SdkResult};
use crate::secret::get_secret;

/// Static model catalog loaded from embedded JSON, wrapped in `RwLock`
/// to support runtime updates via [`merge_models`] and [`refresh`].
///
/// Stores a `Result` to avoid panicking if the embedded JSON is malformed.
/// In practice this cannot fail since the JSON is embedded at compile time,
/// but propagating an error is more consistent with the crate's guidelines.
static CATALOG: LazyLock<Result<RwLock<Vec<ModelInfo>>, String>> = LazyLock::new(|| {
    let json = include_str!("catalog/models.json");
    serde_json::from_str::<Vec<ModelInfo>>(json)
        .map(RwLock::new)
        .map_err(|e| e.to_string())
});

/// Access the catalog lock, mapping a parse failure to `SdkError::Configuration`.
fn catalog() -> SdkResult<&'static RwLock<Vec<ModelInfo>>> {
    CATALOG.as_ref().map_err(|msg| SdkError::Configuration {
        message: format!("catalog/models.json parse error: {msg}"),
    })
}

/// Read-lock the catalog.
pub(crate) fn read_catalog() -> SdkResult<std::sync::RwLockReadGuard<'static, Vec<ModelInfo>>> {
    catalog()?.read().map_err(|e| SdkError::Configuration {
        message: format!("catalog lock poisoned: {e}"),
    })
}

/// Write-lock the catalog.
pub(crate) fn write_catalog() -> SdkResult<std::sync::RwLockWriteGuard<'static, Vec<ModelInfo>>> {
    catalog()?.write().map_err(|e| SdkError::Configuration {
        message: format!("catalog lock poisoned: {e}"),
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
/// Returns `SdkError::Configuration` if the embedded catalog JSON is
/// malformed or the catalog lock is poisoned.
pub fn get_model_info(model_id: &str) -> SdkResult<Option<ModelInfo>> {
    let models = read_catalog()?;
    Ok(models
        .iter()
        .find(|m| m.id == model_id || m.aliases.iter().any(|alias| alias == model_id))
        .cloned())
}

/// List all known models, optionally filtered by provider.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the embedded catalog JSON is
/// malformed or the catalog lock is poisoned.
pub fn list_models(provider: Option<&str>) -> SdkResult<Vec<ModelInfo>> {
    let models = read_catalog()?;
    Ok(match provider {
        Some(p) => models.iter().filter(|m| m.provider == p).cloned().collect(),
        None => models.clone(),
    })
}

/// Check whether a provider's API key, OAuth credential, or equivalent is available.
///
/// If the provider has an entry in `overrides`, it is considered available
/// regardless of environment variables. Pass an empty map when there are no
/// overrides.
#[must_use]
pub fn is_provider_available(provider: &str, overrides: &AuthOverrides) -> bool {
    if overrides.contains_key(provider) {
        return true;
    }
    match provider {
        "openai" => {
            get_secret("OPENAI_API_KEY").is_some() || codex_cli::load_credentials().is_some()
        }
        "anthropic" => {
            get_secret("ANTHROPIC_API_KEY").is_some() || claude_code::load_credentials().is_some()
        }
        "gemini" => {
            get_secret("GEMINI_API_KEY").is_some() || get_secret("GOOGLE_API_KEY").is_some()
        }
        "mistral" => get_secret("MISTRAL_API_KEY").is_some(),
        "deepseek" => get_secret("DEEPSEEK_API_KEY").is_some(),
        "ollama" => {
            std::env::var("OLLAMA_BASE_URL").is_ok() || std::env::var("OLLAMA_HOST").is_ok()
        }
        _ => false,
    }
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
/// Returns `SdkError::Configuration` if the embedded catalog JSON is
/// malformed or the catalog lock is poisoned.
pub fn get_latest_model(provider: &str, capability: Option<&str>) -> SdkResult<Option<ModelInfo>> {
    let models = read_catalog()?;
    Ok(models
        .iter()
        .find(|m| {
            m.provider == provider
                && match capability {
                    None => true,
                    Some("tools") => m.supports_tools,
                    Some("vision") => m.supports_vision,
                    Some("reasoning") => m.supports_reasoning,
                    Some(_) => false, // unknown capability -> no match
                }
        })
        .cloned())
}

/// Merge additional models into the runtime catalog.
///
/// Identity is `(provider, id)` — the same model ID under different
/// providers is treated as distinct entries.
///
/// Models whose `(provider, id)` matches an existing entry are replaced
/// (updated). New models are prepended to the front of their provider's
/// group so they are returned by [`get_latest_model`]. New providers are
/// appended at the end.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the catalog lock is poisoned.
pub fn merge_models(new_models: Vec<ModelInfo>) -> SdkResult<()> {
    let mut models = write_catalog()?;

    for new_model in new_models {
        if let Some(existing) = models
            .iter_mut()
            .find(|m| m.id == new_model.id && m.provider == new_model.provider)
        {
            *existing = new_model;
        } else {
            // Prepend to the front of the provider's group so the new model
            // is treated as "latest". If no models exist for this provider,
            // append at the end (new provider group).
            let insert_pos = models
                .iter()
                .position(|m| m.provider == new_model.provider)
                .unwrap_or(models.len());
            models.insert(insert_pos, new_model);
        }
    }

    Ok(())
}

/// Result of a [`refresh`] operation.
#[derive(Debug, Clone)]
pub struct RefreshResult {
    /// Models that were newly added to the catalog.
    pub new_models: Vec<ModelInfo>,
    /// Per-provider errors encountered during listing.
    /// Each entry is `(provider_name, error)`.
    pub provider_errors: Vec<(String, SdkError)>,
}

/// Refresh the catalog by querying each provider in the client for its
/// available models and merging newly discovered ones.
///
/// Models already in the catalog (by `(provider, id)`) are left
/// unchanged — their curated metadata is preserved. Only models not yet
/// present are added, and they are **appended** after curated entries so
/// that [`get_latest_model`] continues to prefer curated metadata.
///
/// Returns a [`RefreshResult`] containing both newly added models and
/// any per-provider errors, so callers can distinguish "no new models"
/// from "refresh failed everywhere."
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the catalog lock is poisoned.
pub async fn refresh(client: &Client) -> SdkResult<RefreshResult> {
    let mut discovered = Vec::new();
    let mut provider_errors = Vec::new();

    for provider in client.providers() {
        match provider.list_models().await {
            Ok(models) => discovered.extend(models),
            Err(e) => provider_errors.push((provider.name().to_string(), e)),
        }
    }

    // Deduplicate within the discovered set (a provider may list the
    // same model twice), then let append_discovered_models handle the
    // authoritative check under the write lock.
    let mut seen_keys: HashSet<(String, String)> = HashSet::new();
    let candidates: Vec<ModelInfo> = discovered
        .into_iter()
        .filter(|m| seen_keys.insert((m.provider.clone(), m.id.clone())))
        .collect();

    let new_models = if candidates.is_empty() {
        Vec::new()
    } else {
        append_discovered_models(candidates)?
    };

    Ok(RefreshResult {
        new_models,
        provider_errors,
    })
}

/// Append discovered models at the **end** of their provider's group,
/// skipping any that already exist in the catalog.
///
/// Unlike [`merge_models`] (which prepends), this places API-discovered
/// models after curated entries so they don't displace curated "latest"
/// models that have richer metadata.
///
/// The dedup check is performed under the write lock so that concurrent
/// callers cannot insert the same `(provider, id)` pair.
///
/// Returns the models that were actually inserted.
fn append_discovered_models(candidates: Vec<ModelInfo>) -> SdkResult<Vec<ModelInfo>> {
    let mut models = write_catalog()?;
    let mut added = Vec::new();

    for candidate in candidates {
        let already_exists = models
            .iter()
            .any(|m| m.provider == candidate.provider && m.id == candidate.id);
        if already_exists {
            continue;
        }
        // Find the end of this provider's group (one past the last entry
        // for the provider). If no models exist for this provider, append
        // at the very end.
        let insert_pos = models
            .iter()
            .rposition(|m| m.provider == candidate.provider)
            .map_or(models.len(), |last| last + 1);
        models.insert(insert_pos, candidate.clone());
        added.push(candidate);
    }

    Ok(added)
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

    #[test]
    fn merge_adds_new_models() -> Result<(), Box<dyn std::error::Error>> {
        // Use lookup (not count delta) to avoid flakiness from parallel tests
        merge_models(vec![ModelInfo {
            id: "test-merge-add-xyz".into(),
            provider: "test".into(),
            display_name: "Test Model".into(),
            context_window: 4096,
            max_output: None,
            supports_tools: false,
            supports_vision: false,
            supports_reasoning: false,
            input_cost_per_million: None,
            output_cost_per_million: None,
            aliases: vec![],
        }])?;
        let found = get_model_info("test-merge-add-xyz")?;
        assert!(found.is_some(), "merged model should be findable");
        Ok(())
    }

    #[test]
    fn merge_updates_existing() -> Result<(), Box<dyn std::error::Error>> {
        merge_models(vec![ModelInfo {
            id: "test-merge-update-xyz".into(),
            provider: "test".into(),
            display_name: "Original Name".into(),
            context_window: 4096,
            max_output: None,
            supports_tools: false,
            supports_vision: false,
            supports_reasoning: false,
            input_cost_per_million: None,
            output_cost_per_million: None,
            aliases: vec![],
        }])?;

        merge_models(vec![ModelInfo {
            id: "test-merge-update-xyz".into(),
            provider: "test".into(),
            display_name: "Updated Name".into(),
            context_window: 8192,
            max_output: None,
            supports_tools: true,
            supports_vision: false,
            supports_reasoning: false,
            input_cost_per_million: None,
            output_cost_per_million: None,
            aliases: vec![],
        }])?;

        let info =
            get_model_info("test-merge-update-xyz")?.ok_or("model not found after update")?;
        assert_eq!(info.display_name, "Updated Name");
        assert_eq!(info.context_window, 8192);
        assert!(info.supports_tools);
        Ok(())
    }

    #[test]
    fn merge_uses_composite_key() -> Result<(), Box<dyn std::error::Error>> {
        // Same model ID under two different providers should coexist
        merge_models(vec![
            ModelInfo {
                id: "test-composite-key-xyz".into(),
                provider: "provider_a".into(),
                display_name: "Model A".into(),
                context_window: 1024,
                max_output: None,
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                aliases: vec![],
            },
            ModelInfo {
                id: "test-composite-key-xyz".into(),
                provider: "provider_b".into(),
                display_name: "Model B".into(),
                context_window: 2048,
                max_output: None,
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                aliases: vec![],
            },
        ])?;

        // Both should exist
        let a = list_models(Some("provider_a"))?;
        let b = list_models(Some("provider_b"))?;
        assert!(
            a.iter().any(|m| m.id == "test-composite-key-xyz"),
            "provider_a should have the model"
        );
        assert!(
            b.iter().any(|m| m.id == "test-composite-key-xyz"),
            "provider_b should have the model"
        );

        // Updating one should not affect the other
        merge_models(vec![ModelInfo {
            id: "test-composite-key-xyz".into(),
            provider: "provider_a".into(),
            display_name: "Model A Updated".into(),
            context_window: 4096,
            max_output: None,
            supports_tools: false,
            supports_vision: false,
            supports_reasoning: false,
            input_cost_per_million: None,
            output_cost_per_million: None,
            aliases: vec![],
        }])?;

        let b_after = list_models(Some("provider_b"))?;
        let b_model = b_after
            .iter()
            .find(|m| m.id == "test-composite-key-xyz")
            .ok_or("provider_b model disappeared")?;
        assert_eq!(
            b_model.display_name, "Model B",
            "provider_b model should be unchanged"
        );
        Ok(())
    }

    #[test]
    fn merge_prepends_to_provider_group() -> Result<(), Box<dyn std::error::Error>> {
        // Add a model for an existing provider
        let all = list_models(None)?;
        let existing_provider = &all.first().ok_or("catalog is empty")?.provider;

        merge_models(vec![ModelInfo {
            id: "test-prepend-latest-xyz".into(),
            provider: existing_provider.clone(),
            display_name: "Prepended Model".into(),
            context_window: 999_999,
            max_output: None,
            supports_tools: true,
            supports_vision: true,
            supports_reasoning: true,
            input_cost_per_million: None,
            output_cost_per_million: None,
            aliases: vec![],
        }])?;

        // The new model should be returned as "latest" for that provider
        let latest = get_latest_model(existing_provider, None)?.ok_or("no latest model")?;
        assert_eq!(latest.id, "test-prepend-latest-xyz");
        Ok(())
    }
}
