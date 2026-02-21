mod parse;

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use stencila_auth::{AuthOverrides, claude_code, codex_cli};

use crate::client::Client;
use crate::error::{SdkError, SdkResult};
use crate::secret::get_secret;

/// Maps an alias string to the `(provider, model_id)` it resolves to.
pub(crate) type AliasIndex = HashMap<String, (String, String)>;

/// The catalog data: sorted model list and computed alias index.
pub(crate) struct CatalogData {
    pub(crate) models: Vec<ModelInfo>,
    pub(crate) aliases: AliasIndex,
}

/// Static model catalog loaded from embedded JSON, wrapped in `RwLock`
/// to support runtime updates via [`merge_models`] and [`refresh`].
///
/// Stores a `Result` to avoid panicking if the embedded JSON is malformed.
/// In practice this cannot fail since the JSON is embedded at compile time,
/// but propagating an error is more consistent with the crate's guidelines.
static CATALOG: LazyLock<Result<RwLock<CatalogData>, String>> = LazyLock::new(|| {
    let json = include_str!("catalog/models.json");
    let mut models: Vec<ModelInfo> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    sort_provider_groups(&mut models);
    let aliases = compute_aliases(&models);
    Ok(RwLock::new(CatalogData { models, aliases }))
});

/// Access the catalog lock, mapping a parse failure to `SdkError::Configuration`.
fn catalog() -> SdkResult<&'static RwLock<CatalogData>> {
    CATALOG.as_ref().map_err(|msg| SdkError::Configuration {
        message: format!("catalog/models.json parse error: {msg}"),
    })
}

/// Read-lock the catalog.
pub(crate) fn read_catalog() -> SdkResult<std::sync::RwLockReadGuard<'static, CatalogData>> {
    catalog()?.read().map_err(|e| SdkError::Configuration {
        message: format!("catalog lock poisoned: {e}"),
    })
}

/// Write-lock the catalog.
fn write_catalog() -> SdkResult<std::sync::RwLockWriteGuard<'static, CatalogData>> {
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
}

/// Sort each provider group in the catalog so the best model comes first.
///
/// Models are grouped by provider and sorted within each group using a
/// composite key derived from provider-specific ID parsing. The relative
/// ordering of provider groups is preserved.
fn sort_provider_groups(models: &mut [ModelInfo]) {
    // Identify provider group boundaries
    let mut start = 0;
    while start < models.len() {
        let provider = &models[start].provider;
        let end = models[start..]
            .iter()
            .position(|m| m.provider != *provider)
            .map_or(models.len(), |offset| start + offset);
        models[start..end].sort_by_cached_key(|m| parse::sort_key(&m.provider, &m.id, m));
        start = end;
    }
}

#[allow(clippy::too_many_lines)]
/// Build the alias index from the sorted model list.
///
/// Assigns aliases based on sorted position:
/// - **Provider-best**: the first model per provider gets the provider alias
///   (e.g. `"claude"`, `"gpt"`, `"gemini"`)
/// - **Bare-family**: the first model of each family gets the bare alias
///   (e.g. `"opus"`, `"sonnet"`, `"gemini-flash"`)
/// - **Version-pinned**: each model gets version-specific aliases
///   (e.g. `"opus-4.6"`, `"claude-opus-4.6"`)
///
/// Each alias maps to exactly one `(provider, model_id)`. The first model
/// (best-sorted) to claim an alias wins; duplicates are silently skipped.
fn compute_aliases(models: &[ModelInfo]) -> AliasIndex {
    /// Insert an alias only if not already claimed by another model.
    fn try_insert(index: &mut AliasIndex, alias: String, provider: &str, id: &str) {
        index
            .entry(alias)
            .or_insert_with(|| (provider.to_string(), id.to_string()));
    }

    let mut index = AliasIndex::new();

    // Track which bare-family aliases have been assigned per provider
    let mut assigned_families: HashMap<(String, String), bool> = HashMap::new();
    let mut assigned_provider_best: HashSet<String> = HashSet::new();

    for m in models {
        let parsed = parse::parse_model_id(&m.provider, &m.id);

        // Provider-best alias (first model in the provider group)
        if assigned_provider_best.insert(m.provider.clone()) {
            let provider_alias = match m.provider.as_str() {
                "anthropic" => Some("claude"),
                "openai" => {
                    // Only GPT models get the "gpt" alias, not o-series
                    if m.id.starts_with("gpt-") {
                        Some("gpt")
                    } else {
                        None
                    }
                }
                "gemini" => Some("gemini"),
                "mistral" => Some("mistral"),
                _ => None,
            };
            if let Some(alias) = provider_alias {
                try_insert(&mut index, alias.to_string(), &m.provider, &m.id);
            }
        }

        match &parsed {
            parse::ParsedId::Anthropic(p) => {
                let family_key = (m.provider.clone(), p.family.clone());

                // Bare-family aliases
                if let std::collections::hash_map::Entry::Vacant(e) =
                    assigned_families.entry(family_key)
                {
                    e.insert(true);
                    try_insert(&mut index, p.family.clone(), &m.provider, &m.id);
                    try_insert(
                        &mut index,
                        format!("claude-{}", p.family),
                        &m.provider,
                        &m.id,
                    );
                }

                // Version-pinned aliases
                if p.minor > 0 {
                    try_insert(
                        &mut index,
                        format!("{}-{}.{}", p.family, p.major, p.minor),
                        &m.provider,
                        &m.id,
                    );
                    try_insert(
                        &mut index,
                        format!("claude-{}-{}.{}", p.family, p.major, p.minor),
                        &m.provider,
                        &m.id,
                    );
                } else {
                    try_insert(
                        &mut index,
                        format!("{}-{}", p.family, p.major),
                        &m.provider,
                        &m.id,
                    );
                    try_insert(
                        &mut index,
                        format!("claude-{}-{}", p.family, p.major),
                        &m.provider,
                        &m.id,
                    );
                }
            }
            parse::ParsedId::Gpt(p) => {
                let version_str = if p.minor > 0 && p.minor != 100 {
                    format!("{}.{}", p.major, p.minor)
                } else if p.minor == 100 {
                    // 4o special case
                    format!("{}o", p.major)
                } else {
                    format!("{}", p.major)
                };

                let variant_str = p
                    .variant
                    .as_ref()
                    .map(|v| format!("-{v}"))
                    .unwrap_or_default();
                let family_key_str = format!("gpt-{version_str}{variant_str}");
                let family_key = (m.provider.clone(), family_key_str.clone());

                // Bare-family aliases (first of this version+variant combo)
                if !assigned_families.contains_key(&family_key) && p.date.is_some() {
                    // Only assign bare family if this is a dated variant
                    // (the versionless ID like "gpt-5.2" IS the bare alias)
                    assigned_families.insert(family_key, true);
                }

                // Special aliases for variant families
                if let Some(variant) = &p.variant {
                    let variant_family_key = (m.provider.clone(), variant.clone());
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        assigned_families.entry(variant_family_key)
                    {
                        e.insert(true);
                        match variant.as_str() {
                            "codex" => {
                                try_insert(&mut index, "codex".to_string(), &m.provider, &m.id);
                            }
                            "mini" => {
                                try_insert(&mut index, "gpt-mini".to_string(), &m.provider, &m.id);
                            }
                            "nano" => {
                                try_insert(&mut index, "gpt-nano".to_string(), &m.provider, &m.id);
                            }
                            _ => {}
                        }
                    }
                }
            }
            // o-series models are accessed by direct ID (o3, o4-mini, etc.)
            parse::ParsedId::OSeries(_) | parse::ParsedId::Unknown => {}
            parse::ParsedId::Gemini(p) => {
                let family_key = (m.provider.clone(), p.tier.clone());

                // Bare-family aliases
                if let std::collections::hash_map::Entry::Vacant(e) =
                    assigned_families.entry(family_key)
                {
                    e.insert(true);
                    try_insert(&mut index, format!("gemini-{}", p.tier), &m.provider, &m.id);
                    // Also add with sub-tier if it exists
                    if !p.suffix.is_empty()
                        && !p.suffix.contains("preview")
                        && !p.suffix.contains("exp")
                    {
                        let full_tier = format!("{}-{}", p.tier, p.suffix);
                        let sub_family_key = (m.provider.clone(), full_tier.clone());
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            assigned_families.entry(sub_family_key)
                        {
                            e.insert(true);
                            try_insert(
                                &mut index,
                                format!("gemini-{full_tier}"),
                                &m.provider,
                                &m.id,
                            );
                        }
                    }
                } else if !p.suffix.is_empty()
                    && !p.suffix.contains("preview")
                    && !p.suffix.contains("exp")
                {
                    // Assign sub-tier aliases (e.g. "gemini-flash-lite")
                    let full_tier = format!("{}-{}", p.tier, p.suffix);
                    let sub_family_key = (m.provider.clone(), full_tier.clone());
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        assigned_families.entry(sub_family_key)
                    {
                        e.insert(true);
                        try_insert(
                            &mut index,
                            format!("gemini-{full_tier}"),
                            &m.provider,
                            &m.id,
                        );
                    }
                }

                // Version-pinned aliases (first model of each tier+version gets it)
                if p.version_minor > 0 {
                    try_insert(
                        &mut index,
                        format!("gemini-{}-{}.{}", p.tier, p.version_major, p.version_minor),
                        &m.provider,
                        &m.id,
                    );
                } else {
                    try_insert(
                        &mut index,
                        format!("gemini-{}-{}", p.tier, p.version_major),
                        &m.provider,
                        &m.id,
                    );
                }
            }
            parse::ParsedId::Mistral(p) => {
                let family_key = (m.provider.clone(), p.family.clone());

                // For -latest models: strip -latest as alias
                if p.is_latest {
                    try_insert(&mut index, p.family.clone(), &m.provider, &m.id);
                }

                // Bare family aliases (cross sizes) using proper family extraction
                let base_family = parse::mistral_base_family(&p.family);
                let base_key = (m.provider.clone(), base_family.to_string());
                if !assigned_families.contains_key(&base_key) && base_family != p.family {
                    assigned_families.insert(base_key, true);
                    try_insert(&mut index, base_family.to_string(), &m.provider, &m.id);
                }

                assigned_families.entry(family_key).or_insert(true);
            }
        }
    }

    // Remove any alias that exactly matches a model ID (would be confusing)
    let model_ids: HashSet<&str> = models.iter().map(|m| m.id.as_str()).collect();
    index.retain(|alias, _| !model_ids.contains(alias.as_str()));

    index
}

/// Look up a model by its ID or alias. Returns `None` for unknown models.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the embedded catalog JSON is
/// malformed or the catalog lock is poisoned.
pub fn get_model_info(model_id: &str) -> SdkResult<Option<ModelInfo>> {
    let catalog = read_catalog()?;

    // Try direct ID match first
    if let Some(m) = catalog.models.iter().find(|m| m.id == model_id) {
        return Ok(Some(m.clone()));
    }

    // Fall back to alias lookup
    if let Some((provider, id)) = catalog.aliases.get(model_id) {
        return Ok(catalog
            .models
            .iter()
            .find(|m| m.provider == *provider && m.id == *id)
            .cloned());
    }

    Ok(None)
}

/// Return the aliases that resolve to a given model, for display purposes.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the catalog lock is poisoned.
pub fn get_model_aliases(provider: &str, model_id: &str) -> SdkResult<Vec<String>> {
    let catalog = read_catalog()?;
    let mut aliases: Vec<String> = catalog
        .aliases
        .iter()
        .filter(|(_, (p, id))| p == provider && id == model_id)
        .map(|(alias, _)| alias.clone())
        .collect();
    aliases.sort();
    Ok(aliases)
}

/// List all known models, optionally filtered by provider.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if the embedded catalog JSON is
/// malformed or the catalog lock is poisoned.
pub fn list_models(provider: Option<&str>) -> SdkResult<Vec<ModelInfo>> {
    let catalog = read_catalog()?;
    Ok(match provider {
        Some(p) => catalog
            .models
            .iter()
            .filter(|m| m.provider == p)
            .cloned()
            .collect(),
        None => catalog.models.clone(),
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
    let catalog = read_catalog()?;
    Ok(catalog
        .models
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
    let mut catalog = write_catalog()?;

    for new_model in new_models {
        if let Some(existing) = catalog
            .models
            .iter_mut()
            .find(|m| m.id == new_model.id && m.provider == new_model.provider)
        {
            *existing = new_model;
        } else {
            // Prepend to the front of the provider's group so the new model
            // is treated as "latest". If no models exist for this provider,
            // append at the end (new provider group).
            let insert_pos = catalog
                .models
                .iter()
                .position(|m| m.provider == new_model.provider)
                .unwrap_or(catalog.models.len());
            catalog.models.insert(insert_pos, new_model);
        }
    }

    // Re-sort and recompute aliases so new models are correctly ordered
    // and bare-family aliases shift if a new best model appeared.
    sort_provider_groups(&mut catalog.models);
    catalog.aliases = compute_aliases(&catalog.models);

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
    let mut catalog = write_catalog()?;
    let mut added = Vec::new();

    for candidate in candidates {
        let already_exists = catalog
            .models
            .iter()
            .any(|m| m.provider == candidate.provider && m.id == candidate.id);
        if already_exists {
            continue;
        }
        // Find the end of this provider's group (one past the last entry
        // for the provider). If no models exist for this provider, append
        // at the very end.
        let insert_pos = catalog
            .models
            .iter()
            .rposition(|m| m.provider == candidate.provider)
            .map_or(catalog.models.len(), |last| last + 1);
        catalog.models.insert(insert_pos, candidate.clone());
        added.push(candidate);
    }

    if !added.is_empty() {
        sort_provider_groups(&mut catalog.models);
        catalog.aliases = compute_aliases(&catalog.models);
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
        // Use a version-pinned alias that remains stable even when
        // other tests merge new models into the shared catalog.
        let info = get_model_info("opus-4.6")?.ok_or("alias lookup failed")?;
        assert_eq!(info.id, "claude-opus-4-6");
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

    fn test_model(id: &str, provider: &str) -> ModelInfo {
        ModelInfo {
            id: id.into(),
            provider: provider.into(),
            display_name: id.into(),
            context_window: 4096,
            max_output: None,
            supports_tools: false,
            supports_vision: false,
            supports_reasoning: false,
            input_cost_per_million: None,
            output_cost_per_million: None,
        }
    }

    #[test]
    fn merge_adds_new_models() -> Result<(), Box<dyn std::error::Error>> {
        merge_models(vec![test_model("test-merge-add-xyz", "test")])?;
        let found = get_model_info("test-merge-add-xyz")?;
        assert!(found.is_some(), "merged model should be findable");
        Ok(())
    }

    #[test]
    fn merge_updates_existing() -> Result<(), Box<dyn std::error::Error>> {
        merge_models(vec![test_model("test-merge-update-xyz", "test")])?;

        let mut updated = test_model("test-merge-update-xyz", "test");
        updated.display_name = "Updated Name".into();
        updated.context_window = 8192;
        updated.supports_tools = true;
        merge_models(vec![updated])?;

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
            test_model("test-composite-key-xyz", "provider_a"),
            test_model("test-composite-key-xyz", "provider_b"),
        ])?;

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
        let mut updated = test_model("test-composite-key-xyz", "provider_a");
        updated.display_name = "Model A Updated".into();
        merge_models(vec![updated])?;

        let b_after = list_models(Some("provider_b"))?;
        let b_model = b_after
            .iter()
            .find(|m| m.id == "test-composite-key-xyz")
            .ok_or("provider_b model disappeared")?;
        assert_eq!(
            b_model.display_name, "test-composite-key-xyz",
            "provider_b model should be unchanged"
        );
        Ok(())
    }

    #[test]
    fn merge_inserts_into_provider_group() -> Result<(), Box<dyn std::error::Error>> {
        merge_models(vec![test_model("test-merge-insert-xyz", "test_merge_sort")])?;
        let found = get_model_info("test-merge-insert-xyz")?;
        assert!(found.is_some(), "merged model should be findable");
        Ok(())
    }

    // --- Sort tests ---

    #[test]
    fn sort_promotes_best_openai() -> Result<(), Box<dyn std::error::Error>> {
        let openai = list_models(Some("openai"))?;
        let first = openai.first().ok_or("no openai models")?;
        assert!(
            first.id.starts_with("gpt-5"),
            "first OpenAI model should be gpt-5.x, got {}",
            first.id
        );
        Ok(())
    }

    #[test]
    fn sort_preserves_anthropic_order() -> Result<(), Box<dyn std::error::Error>> {
        let anthropic = list_models(Some("anthropic"))?;
        let ids: Vec<&str> = anthropic.iter().map(|m| m.id.as_str()).collect();
        let opus_pos = ids
            .iter()
            .position(|id| id.contains("opus"))
            .ok_or("no opus")?;
        let sonnet_pos = ids
            .iter()
            .position(|id| id.contains("sonnet"))
            .ok_or("no sonnet")?;
        let haiku_pos = ids
            .iter()
            .position(|id| id.contains("haiku"))
            .ok_or("no haiku")?;
        assert!(
            opus_pos < sonnet_pos,
            "opus ({opus_pos}) should come before sonnet ({sonnet_pos})"
        );
        assert!(
            sonnet_pos < haiku_pos,
            "sonnet ({sonnet_pos}) should come before haiku ({haiku_pos})"
        );
        Ok(())
    }

    #[test]
    fn get_latest_model_returns_best() -> Result<(), Box<dyn std::error::Error>> {
        let claude = get_latest_model("anthropic", None)?.ok_or("no anthropic latest")?;
        assert!(
            claude.id.contains("opus"),
            "latest anthropic should be opus, got {}",
            claude.id
        );

        let gpt = get_latest_model("openai", None)?.ok_or("no openai latest")?;
        assert!(
            gpt.id.starts_with("gpt-5"),
            "latest openai should be gpt-5.x, got {}",
            gpt.id
        );

        let gemini = get_latest_model("gemini", None)?.ok_or("no gemini latest")?;
        assert!(
            gemini.id.contains("pro"),
            "latest gemini should be pro, got {}",
            gemini.id
        );
        Ok(())
    }

    // --- Alias tests ---

    #[test]
    fn computed_aliases_match_expected() -> Result<(), Box<dyn std::error::Error>> {
        let claude = get_model_info("claude")?.ok_or("claude alias not found")?;
        assert!(
            claude.id.contains("opus"),
            "claude alias should point to opus, got {}",
            claude.id
        );

        let opus = get_model_info("opus")?.ok_or("opus alias not found")?;
        assert!(opus.id.contains("opus"), "opus alias wrong: {}", opus.id);

        let sonnet = get_model_info("sonnet")?.ok_or("sonnet alias not found")?;
        assert!(
            sonnet.id.contains("sonnet"),
            "sonnet alias wrong: {}",
            sonnet.id
        );

        let haiku = get_model_info("haiku")?.ok_or("haiku alias not found")?;
        assert!(
            haiku.id.contains("haiku"),
            "haiku alias wrong: {}",
            haiku.id
        );

        let gemini = get_model_info("gemini")?.ok_or("gemini alias not found")?;
        assert!(
            gemini.id.contains("pro"),
            "gemini alias should point to pro, got {}",
            gemini.id
        );

        let flash = get_model_info("gemini-flash")?.ok_or("gemini-flash alias not found")?;
        assert!(
            flash.id.contains("flash"),
            "gemini-flash alias wrong: {}",
            flash.id
        );

        Ok(())
    }

    #[test]
    fn version_pinned_aliases_stable() -> Result<(), Box<dyn std::error::Error>> {
        let opus45 = get_model_info("opus-4.5")?.ok_or("opus-4.5 alias not found")?;
        assert!(
            opus45.id.starts_with("claude-opus-4-5"),
            "opus-4.5 should resolve to claude-opus-4-5-*, got {}",
            opus45.id
        );

        let opus46 = get_model_info("opus-4.6")?.ok_or("opus-4.6 alias not found")?;
        assert_eq!(opus46.id, "claude-opus-4-6");

        Ok(())
    }

    #[test]
    fn bare_alias_shifts_on_merge() -> Result<(), Box<dyn std::error::Error>> {
        let mut model = test_model("claude-opus-5-0", "anthropic");
        model.context_window = 500_000;
        model.max_output = Some(256_000);
        model.supports_tools = true;
        model.supports_vision = true;
        model.supports_reasoning = true;
        model.input_cost_per_million = Some(10.0);
        model.output_cost_per_million = Some(50.0);
        merge_models(vec![model])?;

        let claude = get_model_info("claude")?.ok_or("claude alias not found after merge")?;
        assert_eq!(
            claude.id, "claude-opus-5-0",
            "claude alias should shift to new best opus"
        );

        let opus = get_model_info("opus")?.ok_or("opus alias not found after merge")?;
        assert_eq!(opus.id, "claude-opus-5-0");

        Ok(())
    }

    #[test]
    fn aliases_not_in_json() {
        let json = include_str!("catalog/models.json");
        assert!(
            !json.contains("\"aliases\""),
            "models.json should not contain aliases field"
        );
    }

    #[test]
    fn get_model_aliases_returns_aliases() -> Result<(), Box<dyn std::error::Error>> {
        let aliases = get_model_aliases("anthropic", "claude-opus-4-6")?;
        assert!(
            aliases.contains(&"opus-4.6".to_string()),
            "should contain version-pinned alias, got: {aliases:?}"
        );
        assert!(
            aliases.contains(&"claude-opus-4.6".to_string()),
            "should contain prefixed alias, got: {aliases:?}"
        );
        Ok(())
    }
}
