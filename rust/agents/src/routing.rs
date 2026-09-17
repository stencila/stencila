//! Session routing: decides whether an agent session should use an API
//! backend or fall back to a CLI tool.
//!
//! The core entry point is [`route_session`], which inspects the agent's
//! provider/model fields and the available API credentials to produce a
//! [`SessionRoute`]. Use [`route_session_explained`] to obtain a full
//! [`RoutingDecision`] with metadata about *why* the route was chosen.

use std::fmt;

use crate::error::{AgentError, AgentResult};

/// CLI provider names that delegate to external CLI tools.
const CLI_PROVIDERS: &[&str] = &["claude-cli", "codex-cli", "gemini-cli"];

/// Whether a provider name refers to a CLI provider.
pub fn is_cli_provider(provider: &str) -> bool {
    CLI_PROVIDERS.contains(&provider)
}

/// Map an API provider name to its corresponding CLI provider name.
///
/// Used for fallback: when no API key is available for a provider, the
/// session can be routed to the matching CLI tool instead.
pub fn api_to_cli(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("claude-cli"),
        "openai" => Some("codex-cli"),
        "gemini" | "google" => Some("gemini-cli"),
        _ => None,
    }
}

/// Map a CLI provider name back to its API provider name.
pub fn cli_to_api(provider: &str) -> Option<&'static str> {
    match provider {
        "claude-cli" => Some("anthropic"),
        "codex-cli" => Some("openai"),
        "gemini-cli" => Some("gemini"),
        _ => None,
    }
}

/// Default model alias for each API provider when none is specified.
///
/// Returns the alias that points to the latest model for that provider.
/// For providers with dynamic model lists (e.g. Ollama), use
/// [`resolve_default_model`] instead.
fn default_model(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("claude"),
        "openai" => Some("gpt"),
        "gemini" | "google" => Some("gemini"),
        "mistral" => Some("mistral-large-latest"),
        _ => None,
    }
}

/// Resolve the default model for a provider, supporting both static aliases
/// and dynamic providers like Ollama.
///
/// For Ollama, checks (in order):
/// 1. `[models.ollama].default_model` in `stencila.toml`
/// 2. The first model in the catalog with `provider == "ollama"`
///    (populated by live refresh or auto-query)
///
/// Falls back to [`default_model`] for all other providers.
fn resolve_default_model(provider: &str) -> Option<String> {
    if let Some(alias) = default_model(provider) {
        return Some(alias.to_string());
    }

    if provider == "ollama" {
        // Check config for a user-specified default
        if let Some(model) = ollama_default_model_from_config() {
            return Some(model);
        }

        // Try the catalog (populated by auto-query on detection)
        if let Ok(models) = stencila_models3::catalog::list_models(Some("ollama")) {
            if let Some(first) = models.first() {
                return Some(first.id.clone());
            }
        }
    }

    None
}

/// Read `[models.ollama].default_model` from stencila.toml config.
fn ollama_default_model_from_config() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let config = stencila_config::load_and_validate(&cwd).ok()?;
    config
        .models
        .and_then(|m| m.ollama)
        .and_then(|o| o.default_model)
}

/// Hint for which environment variable to set for a given provider.
fn api_key_env_hint(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "gemini" | "google" => "GEMINI_API_KEY",
        "mistral" => "MISTRAL_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
        "ollama" => "OLLAMA_BASE_URL (or run `ollama serve`)",
        _ => "<PROVIDER>_API_KEY",
    }
}

/// The routing decision for a session: use an API backend or fall back to a
/// CLI tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRoute {
    /// Use the LLM API directly.
    Api { provider: String, model: String },
    /// Delegate to an external CLI tool.
    Cli {
        provider: String,
        model: Option<String>,
    },
}

impl fmt::Display for SessionRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api { provider, model } => write!(f, "{provider} / {model} (API)"),
            Self::Cli { provider, model } => {
                if let Some(m) = model {
                    write!(f, "{provider} / {m} (CLI)")
                } else {
                    write!(f, "{provider} (CLI)")
                }
            }
        }
    }
}

/// Where the provider came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderSource {
    /// Explicitly set in the agent definition (AGENT.md header).
    AgentExplicit,
    /// Inferred from the model name via catalog or heuristics.
    InferredFromModel,
    /// Selected from the model catalog based on model size preference.
    CatalogModelSize,
    /// Selected as the first available configured API provider.
    DefaultConfig,
    /// Fell back to a CLI tool because no API providers were available.
    CliDefault,
}

impl fmt::Display for ProviderSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentExplicit => write!(f, "provider from agent definition"),
            Self::InferredFromModel => write!(f, "provider inferred from model"),
            Self::CatalogModelSize => write!(f, "provider selected by model size from catalog"),
            Self::DefaultConfig => write!(f, "default configured provider"),
            Self::CliDefault => write!(f, "CLI fallback (no API providers)"),
        }
    }
}

/// Where the model came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSource {
    /// Explicitly set in the agent definition.
    AgentExplicit,
    /// Selected from the model catalog based on model size preference.
    CatalogModelSize,
    /// Default alias for the resolved provider.
    DefaultAlias,
    /// CLI tool's own default (no model passed).
    CliDefault,
    /// Auto-selected fallback replacing an incompatible model.
    Fallback,
}

/// How the model was selected by the routing logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionMechanism {
    /// Selected from an explicit list of model IDs (highest priority).
    ExplicitList {
        /// Zero-based index into the models list.
        index: usize,
        /// The model ID that was selected.
        id: String,
    },
    /// Selected by model size preference.
    ModelSize {
        /// The requested size (e.g. "small", "medium", "large").
        size: String,
    },
    /// Selected based on provider preference list.
    ProviderPreference,
    /// Fell through to the default selection path.
    Default,
}

/// Why a candidate model was skipped during selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The model's inferred provider has no API credentials configured.
    NoCredentials {
        /// The provider that lacked credentials.
        provider: String,
    },
    /// The model ID was not found in the catalog.
    NotInCatalog {
        /// The model ID that was not found.
        model: String,
    },
    /// No provider could be inferred for the model.
    NoProviderInferred {
        /// The model ID for which no provider was inferred.
        model: String,
    },
}

/// A model that was considered but skipped during routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedModel {
    /// The model ID that was skipped.
    pub model: String,
    /// Why it was skipped.
    pub reason: SkipReason,
}

impl fmt::Display for ModelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentExplicit => write!(f, "from agent definition"),
            Self::CatalogModelSize => write!(f, "selected by model size from catalog"),
            Self::DefaultAlias => write!(f, "default alias for provider"),
            Self::CliDefault => write!(f, "CLI default"),
            Self::Fallback => write!(f, "auto-selected fallback"),
        }
    }
}

/// Full routing decision with explanation metadata.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// The resolved route (API or CLI).
    pub route: SessionRoute,
    /// Where the provider was determined from.
    pub provider_source: ProviderSource,
    /// Where the model was determined from.
    pub model_source: ModelSource,
    /// If the model name was an alias, the resolved concrete model ID.
    /// `(alias, concrete_id)`.
    pub alias_resolution: Option<(String, String)>,
    /// Whether an API→CLI fallback occurred.
    pub fallback_used: bool,
    /// Human-readable reason for the fallback, if any.
    pub fallback_reason: Option<String>,
    /// How the model was selected.
    pub selection_mechanism: SelectionMechanism,
    /// Models that were considered but skipped, with reasons.
    pub skipped: Vec<SkippedModel>,
}

impl RoutingDecision {
    /// Format as a concise one-line summary for user display.
    pub fn summary(&self) -> String {
        let (backend, provider, model) = match &self.route {
            SessionRoute::Api { provider, model } => {
                ("API", provider.as_str(), Some(model.as_str()))
            }
            SessionRoute::Cli { provider, model } => ("CLI", provider.as_str(), model.as_deref()),
        };

        let model_display = if let Some((alias, concrete)) = &self.alias_resolution {
            format!("{alias}→{concrete}")
        } else if let Some(m) = model {
            m.to_string()
        } else {
            "default".to_string()
        };

        let mechanism = match &self.selection_mechanism {
            SelectionMechanism::ExplicitList { index, .. } => {
                format!("preferred model #{index}")
            }
            SelectionMechanism::ModelSize { size } => format!("modelSize={size}"),
            SelectionMechanism::ProviderPreference => "provider preference".to_string(),
            SelectionMechanism::Default => "default selection".to_string(),
        };

        let skipped = if self.skipped.is_empty() {
            String::new()
        } else {
            format!("; skipped {}", self.skipped.len())
        };

        format!(
            "Using {provider}/{model_display} ({backend}; {provider_source}; {mechanism}{skipped})",
            provider_source = self.provider_source,
        )
    }

    /// Format the fallback warning, if a fallback occurred.
    pub fn fallback_warning(&self) -> Option<String> {
        if !self.fallback_used {
            return None;
        }
        let reason = self
            .fallback_reason
            .as_deref()
            .unwrap_or("no API credentials");
        match &self.route {
            SessionRoute::Cli { provider, .. } => Some(if self.skipped.is_empty() {
                format!("Falling back to {provider} - {reason}")
            } else {
                format!(
                    "Falling back to {provider} - {reason}; skipped: {}",
                    format_skipped_models(&self.skipped)
                )
            }),
            SessionRoute::Api { model, .. } => Some(if self.skipped.is_empty() {
                format!("Using fallback model {model} - {reason}")
            } else {
                format!(
                    "Using fallback model {model} - {reason}; skipped: {}",
                    format_skipped_models(&self.skipped)
                )
            }),
        }
    }
}

/// Decide whether to use an API or CLI backend for the given agent.
///
/// The decision follows a strict priority order:
///
/// 1. **Explicit model list** (`models`) — iterate the list, infer the
///    provider for each, check credentials, and select the first available.
/// 2. **Model size** (`model_size`) — query the catalog for models of the
///    requested size, optionally filtered by `providers`.
/// 3. **Provider preference** (`providers`) — iterate providers in order,
///    select the first with credentials, and use its default model.
/// 4. **Default** — fall back to the first configured API provider or a
///    CLI tool on PATH.
///
/// Within each path, if no API credentials are available for the chosen
/// provider, the router attempts a CLI fallback (e.g. `anthropic` →
/// `claude-cli`). If no CLI mapping exists, an error is returned.
///
/// This is a thin wrapper around [`route_session_explained`] that discards
/// the explanation metadata.
pub fn route_session(
    models: Option<&[String]>,
    providers: Option<&[String]>,
    model_size: Option<&str>,
    client: &stencila_models3::client::Client,
) -> AgentResult<SessionRoute> {
    Ok(route_session_explained(models, providers, model_size, client)?.route)
}

/// Like [`route_session`] but returns a full [`RoutingDecision`] with
/// explanation metadata for UX surfaces.
pub fn route_session_explained(
    models: Option<&[String]>,
    providers: Option<&[String]>,
    model_size: Option<&str>,
    client: &stencila_models3::client::Client,
) -> AgentResult<RoutingDecision> {
    // Single model+provider pair — use the direct routing path.
    if let (Some([model]), Some([provider])) = (models, providers)
        && !model.eq_ignore_ascii_case("any")
        && !provider.eq_ignore_ascii_case("any")
    {
        return route_direct(Some(provider), Some(model), client);
    }

    // Priority 1: Explicit model list
    if let Some(model_list) = models
        && !model_list.is_empty()
        && let Some(decision) = route_via_models_path(model_list, client)?
    {
        return Ok(decision);
    }

    // Priority 2: Model size preference
    if let Some(size_str) = model_size
        && let Some(size) = parse_model_size(size_str)
        && let Some(decision) = route_via_model_size_path(size_str, size, providers, client)?
    {
        return Ok(decision);
    }
    // No candidates found for this size / unrecognized size — fall through.

    // Priority 3: Provider preference list
    if let Some(provider_list) = providers
        && !provider_list.is_empty()
        && let Some(decision) = route_via_providers_path(provider_list, client)?
    {
        return Ok(decision);
    }
    // No provider in the list has credentials — fall through to default.

    // Priority 4: Default path
    route_direct(None, None, client)
}

/// Route using the explicit models list (highest priority path).
///
/// Iterates the model list in order, infers the provider for each,
/// checks for API credentials, and selects the first model whose
/// provider has credentials. Models that cannot be used are tracked
/// in the `skipped` list with a reason.
fn route_via_models_path(
    model_list: &[String],
    client: &stencila_models3::client::Client,
) -> AgentResult<Option<RoutingDecision>> {
    let mut skipped = Vec::new();
    let allow_fallback = model_list
        .iter()
        .any(|model_id| model_id.eq_ignore_ascii_case("any"));

    for (index, model_id) in model_list.iter().enumerate() {
        if model_id.eq_ignore_ascii_case("any") {
            continue;
        }

        let provider = match resolve_catalog_model(model_id) {
            Ok((provider, _resolved_model)) => provider,
            Err(SkipReason::NotInCatalog { .. }) => match infer_provider(model_id, client) {
                Ok(provider) => canonical_provider(&provider).to_string(),
                Err(reason) => {
                    skipped.push(SkippedModel {
                        model: model_id.clone(),
                        reason,
                    });
                    continue;
                }
            },
            Err(reason) => {
                skipped.push(SkippedModel {
                    model: model_id.clone(),
                    reason,
                });
                continue;
            }
        };

        // Check if client has credentials for this provider
        if client.has_provider(&provider) {
            let (model, fallback_used, fallback_reason) =
                resolve_auth_compatible_model(&provider, model_id, client)?;
            return Ok(Some(RoutingDecision {
                route: SessionRoute::Api { provider, model },
                provider_source: ProviderSource::AgentExplicit,
                model_source: if fallback_used {
                    ModelSource::Fallback
                } else {
                    ModelSource::AgentExplicit
                },
                alias_resolution: resolve_model_alias(model_id),
                fallback_used,
                fallback_reason,
                selection_mechanism: SelectionMechanism::ExplicitList {
                    index,
                    id: model_id.clone(),
                },
                skipped,
            }));
        }

        // No credentials for this provider — record skip and continue
        skipped.push(SkippedModel {
            model: model_id.clone(),
            reason: SkipReason::NoCredentials {
                provider: provider.clone(),
            },
        });
    }

    // All models exhausted — try CLI fallback for the first model that had
    // a known provider.
    for skipped_model in &skipped {
        if let SkipReason::NoCredentials { ref provider } = skipped_model.reason
            && let Some(cli) = api_to_cli(provider)
        {
            return Ok(Some(RoutingDecision {
                route: SessionRoute::Cli {
                    provider: cli.to_string(),
                    model: Some(skipped_model.model.clone()),
                },
                provider_source: ProviderSource::AgentExplicit,
                model_source: ModelSource::AgentExplicit,
                alias_resolution: None,
                fallback_used: true,
                fallback_reason: Some(format!(
                    "No API key for {provider} (set {})",
                    api_key_env_hint(provider)
                )),
                selection_mechanism: SelectionMechanism::ExplicitList {
                    index: model_list
                        .iter()
                        .position(|model| model == &skipped_model.model)
                        .unwrap_or(0),
                    id: skipped_model.model.clone(),
                },
                skipped: skipped.clone(),
            }));
        }
    }

    if allow_fallback {
        return Ok(None);
    }

    // Build a descriptive error
    let model_names: Vec<&str> = model_list
        .iter()
        .filter(|model| !model.eq_ignore_ascii_case("any"))
        .map(String::as_str)
        .collect();
    Err(AgentError::Sdk(
        stencila_models3::error::SdkError::Configuration {
            message: format!(
                "No available provider for model(s): {}. \
                 None of the listed models have API credentials configured \
                 and no CLI fallback is available. \
                 Tip: add 'any' to the models list to allow fallback to \
                 modelSize, providers, or defaults.",
                model_names.join(", ")
            ),
        },
    ))
}

/// Parse a model size string into a [`ModelSize`] enum value.
///
/// Case-insensitive matching for "large", "medium", and "small".
/// Returns `None` for unrecognized values.
pub fn parse_model_size(s: &str) -> Option<stencila_models3::catalog::ModelSize> {
    use stencila_models3::catalog::ModelSize;

    match s.to_ascii_lowercase().as_str() {
        "large" => Some(ModelSize::Large),
        "medium" => Some(ModelSize::Medium),
        "small" => Some(ModelSize::Small),
        _ => None,
    }
}

/// Route using model size preference (second-priority path).
///
/// Queries the catalog for models of the requested size, optionally
/// filtered by the provider list, and selects the first model whose
/// provider has credentials in the client. Returns `Ok(None)` when the
/// requested size cannot be satisfied so the caller can fall through to
/// provider preference or default routing.
fn route_via_model_size_path(
    size_str: &str,
    size: stencila_models3::catalog::ModelSize,
    providers: Option<&[String]>,
    client: &stencila_models3::client::Client,
) -> AgentResult<Option<RoutingDecision>> {
    let provider_filter = providers.map(canonicalize_provider_list);
    let mut candidates = Vec::new();

    if let Some(ref provider_list) = provider_filter {
        for provider in provider_list {
            candidates.extend(
                stencila_models3::catalog::list_models_by_size(size, Some(provider))
                    .map_err(AgentError::Sdk)?,
            );
        }
    } else {
        candidates =
            stencila_models3::catalog::list_models_by_size(size, None).map_err(AgentError::Sdk)?;
    }

    for candidate in &candidates {
        // Check if client has credentials for this provider
        if client.has_provider(&candidate.provider) {
            let (model, fallback_used, fallback_reason) =
                resolve_auth_compatible_model(&candidate.provider, &candidate.id, client)?;
            return Ok(Some(RoutingDecision {
                route: SessionRoute::Api {
                    provider: candidate.provider.clone(),
                    model,
                },
                provider_source: ProviderSource::CatalogModelSize,
                model_source: if fallback_used {
                    ModelSource::Fallback
                } else {
                    ModelSource::CatalogModelSize
                },
                alias_resolution: None,
                fallback_used,
                fallback_reason,
                selection_mechanism: SelectionMechanism::ModelSize {
                    size: size_str.to_string(),
                },
                skipped: Vec::new(),
            }));
        }
    }

    Ok(None)
}

/// Route using provider preference list (third-priority path).
///
/// Iterates the provider list in order, selects the first provider with
/// credentials, and uses its default model.
fn route_via_providers_path(
    provider_list: &[String],
    client: &stencila_models3::client::Client,
) -> AgentResult<Option<RoutingDecision>> {
    let allow_fallback = provider_list
        .iter()
        .any(|provider| provider.eq_ignore_ascii_case("any"));
    let providers = provider_list
        .iter()
        .filter(|provider| !provider.eq_ignore_ascii_case("any"))
        .map(|provider| canonical_provider(provider).to_string())
        .collect::<Vec<_>>();

    for provider in &providers {
        if !client.has_provider(provider) {
            continue;
        }

        let Some(model_alias) = resolve_default_model(provider) else {
            continue;
        };
        let (model, fallback_used, fallback_reason) =
            resolve_auth_compatible_model(provider, &model_alias, client)?;

        return Ok(Some(RoutingDecision {
            route: SessionRoute::Api {
                provider: provider.clone(),
                model,
            },
            provider_source: ProviderSource::AgentExplicit,
            model_source: if fallback_used {
                ModelSource::Fallback
            } else {
                ModelSource::DefaultAlias
            },
            alias_resolution: None,
            fallback_used,
            fallback_reason,
            selection_mechanism: SelectionMechanism::ProviderPreference,
            skipped: Vec::new(),
        }));
    }

    for provider in &providers {
        if let Some(cli) = api_to_cli(provider) {
            return Ok(Some(RoutingDecision {
                route: SessionRoute::Cli {
                    provider: cli.to_string(),
                    model: resolve_default_model(provider),
                },
                provider_source: ProviderSource::AgentExplicit,
                model_source: ModelSource::DefaultAlias,
                alias_resolution: None,
                fallback_used: true,
                fallback_reason: Some(format!(
                    "No API key for {provider} (set {})",
                    api_key_env_hint(provider)
                )),
                selection_mechanism: SelectionMechanism::ProviderPreference,
                skipped: Vec::new(),
            }));
        }
    }

    if allow_fallback {
        return Ok(None);
    }

    Err(AgentError::Sdk(
        stencila_models3::error::SdkError::Configuration {
            message: format!(
                "None of the preferred providers are configured: {}. \
                 Tip: add 'any' to the providers list to allow fallback to \
                 other configured providers.",
                providers.to_vec().join(", ")
            ),
        },
    ))
}

/// Infer the provider for a model by trying, in order: client inference,
/// catalog lookup, and name-based heuristics. Returns `Err(SkipReason)`
/// when no provider can be determined.
fn infer_provider(
    model_id: &str,
    client: &stencila_models3::client::Client,
) -> Result<String, SkipReason> {
    // 1. Client-level inference (provider adapters)
    if let Ok(Some(p)) = client.infer_provider_from_model(model_id) {
        return Ok(p);
    }

    // 2. Catalog lookup
    if let Ok(Some(info)) = stencila_models3::catalog::get_model_info(model_id) {
        return Ok(info.provider);
    }

    // 3. Name-based heuristic
    if let Some(p) = infer_provider_heuristic(model_id) {
        return Ok(p.to_string());
    }

    Err(SkipReason::NoProviderInferred {
        model: model_id.to_string(),
    })
}

fn resolve_catalog_model(model_id: &str) -> Result<(String, String), SkipReason> {
    let info = stencila_models3::catalog::get_model_info(model_id).map_err(|_| {
        SkipReason::NotInCatalog {
            model: model_id.to_string(),
        }
    })?;
    let info = info.ok_or_else(|| SkipReason::NotInCatalog {
        model: model_id.to_string(),
    })?;
    Ok((canonical_provider(&info.provider).to_string(), info.id))
}

fn canonical_provider(provider: &str) -> &str {
    match provider {
        "google" => "gemini",
        other => other,
    }
}

fn canonicalize_provider_list(providers: &[String]) -> Vec<String> {
    providers
        .iter()
        .map(|provider| canonical_provider(provider).to_string())
        .collect()
}

fn format_skipped_models(skipped: &[SkippedModel]) -> String {
    skipped
        .iter()
        .map(|skipped| match &skipped.reason {
            SkipReason::NoCredentials { provider } => {
                format!("{} (no credentials for {provider})", skipped.model)
            }
            SkipReason::NotInCatalog { .. } => format!("{} (not in catalog)", skipped.model),
            SkipReason::NoProviderInferred { .. } => {
                format!("{} (no provider inferred)", skipped.model)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Infer a provider from a model ID using name-based heuristics.
fn infer_provider_heuristic(model_id: &str) -> Option<&'static str> {
    if model_id.starts_with("claude") {
        Some("anthropic")
    } else if model_id.starts_with("gpt-")
        || model_id.starts_with("o1")
        || model_id.starts_with("o3")
        || model_id.starts_with("o4")
    {
        Some("openai")
    } else if model_id.starts_with("gemini") {
        Some("gemini")
    } else if model_id.starts_with("mistral") {
        Some("mistral")
    } else if model_id.starts_with("deepseek") {
        Some("deepseek")
    } else {
        None
    }
}

/// Route a single provider/model pair through the full resolution pipeline.
///
/// Handles explicit CLI providers, API auth checking with auth-type-aware
/// fallback, provider inference from model names, default model selection,
/// and API→CLI fallback when no API credentials are available. Also serves
/// as the final default path when no model list, size, or provider
/// preferences are specified.
pub fn route_direct(
    agent_provider: Option<&str>,
    agent_model: Option<&str>,
    client: &stencila_models3::client::Client,
) -> AgentResult<RoutingDecision> {
    // 1. Explicit CLI provider
    if let Some(p) = agent_provider
        && is_cli_provider(p)
    {
        let model_source = if agent_model.is_some() {
            ModelSource::AgentExplicit
        } else {
            ModelSource::CliDefault
        };
        let alias_resolution = agent_model.and_then(resolve_model_alias);
        return Ok(RoutingDecision {
            route: SessionRoute::Cli {
                provider: p.to_string(),
                model: agent_model.map(String::from),
            },
            provider_source: ProviderSource::AgentExplicit,
            model_source,
            alias_resolution,
            fallback_used: false,
            fallback_reason: None,
            selection_mechanism: SelectionMechanism::Default,
            skipped: Vec::new(),
        });
    }

    // 2. Resolve API provider + model
    let (api_provider, model, provider_source, model_source) = match (agent_provider, agent_model) {
        (Some(p), Some(m)) => (
            p.to_string(),
            m.to_string(),
            ProviderSource::AgentExplicit,
            ModelSource::AgentExplicit,
        ),
        (Some(p), None) => {
            let m = resolve_default_model(p).ok_or_else(|| {
                AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                    message: format!(
                        "No default model for provider '{p}'. \
                                 Please specify a model explicitly."
                    ),
                })
            })?;
            (
                p.to_string(),
                m,
                ProviderSource::AgentExplicit,
                ModelSource::DefaultAlias,
            )
        }
        (None, Some(m)) => {
            let p = client
                .infer_provider_from_model(m)
                .map_err(AgentError::Sdk)?
                .ok_or_else(|| {
                    AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                        message: format!(
                            "Cannot infer provider for model '{m}'. \
                                 Please specify the provider explicitly."
                        ),
                    })
                })?;
            (
                p,
                m.to_string(),
                ProviderSource::InferredFromModel,
                ModelSource::AgentExplicit,
            )
        }
        (None, None) => {
            if let Some(p) = client.select_provider() {
                let m = resolve_default_model(p).ok_or_else(|| {
                    AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                        message: format!(
                            "No default model for provider '{p}'. \
                                     Please specify a model explicitly."
                        ),
                    })
                })?;
                (
                    p.to_string(),
                    m,
                    ProviderSource::DefaultConfig,
                    ModelSource::DefaultAlias,
                )
            } else {
                // No API providers — fall back to first available CLI tool
                let cli_provider = detect_cli_provider().ok_or_else(|| {
                    AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                        message: "No API providers configured and no supported CLI tool \
                                      found on PATH.\n\
                                      Install one of: claude, codex, gemini — \
                                      or set an API key (e.g. ANTHROPIC_API_KEY, OPENAI_API_KEY)."
                            .to_string(),
                    })
                })?;
                return Ok(RoutingDecision {
                    route: SessionRoute::Cli {
                        provider: cli_provider,
                        model: None,
                    },
                    provider_source: ProviderSource::CliDefault,
                    model_source: ModelSource::CliDefault,
                    alias_resolution: None,
                    fallback_used: false,
                    fallback_reason: None,
                    selection_mechanism: SelectionMechanism::Default,
                    skipped: Vec::new(),
                });
            }
        }
    };

    let alias_resolution = resolve_model_alias(&model);

    // 3. API auth available — use API
    if client.has_provider(&api_provider) {
        let (selected_model, fallback_used, fallback_reason) =
            resolve_auth_compatible_model(&api_provider, &model, client)?;

        return Ok(RoutingDecision {
            route: SessionRoute::Api {
                provider: api_provider,
                model: selected_model,
            },
            provider_source,
            model_source: if fallback_used {
                ModelSource::Fallback
            } else {
                model_source
            },
            alias_resolution: if fallback_used {
                None
            } else {
                alias_resolution
            },
            fallback_used,
            fallback_reason,
            selection_mechanism: SelectionMechanism::Default,
            skipped: Vec::new(),
        });
    }

    // 4. No auth — try mapped CLI fallback
    if let Some(cli) = api_to_cli(&api_provider) {
        let env_hint = api_key_env_hint(&api_provider);
        return Ok(RoutingDecision {
            route: SessionRoute::Cli {
                provider: cli.to_string(),
                model: Some(model),
            },
            provider_source,
            model_source,
            alias_resolution,
            fallback_used: true,
            fallback_reason: Some(format!("No API key for {api_provider} (set {env_hint})")),
            selection_mechanism: SelectionMechanism::Default,
            skipped: Vec::new(),
        });
    }

    // 5. No mapping — error
    Err(AgentError::Sdk(
        stencila_models3::error::SdkError::Configuration {
            message: format!(
                "Provider '{api_provider}' is not configured. \
                 Set the appropriate API key (e.g. {}).",
                api_key_env_hint(&api_provider)
            ),
        },
    ))
}

/// Resolve a model identifier to a concrete, auth-compatible model ID.
///
/// If `model` is an alias (e.g. `"claude"`, `"gpt"`), it is first resolved
/// to the concrete catalog ID (e.g. `"claude-opus-4-6"`, `"gpt-5.4-pro"`).
/// The concrete ID is then checked against the provider's authentication
/// type. When the model is incompatible with the current credentials, a
/// same-family fallback is attempted.
///
/// Returns `(concrete_model_id, fallback_used, fallback_reason)`. The
/// returned model ID is always fully resolved — never an alias — so that
/// downstream consumers (profiles, session records, audit logs) record the
/// actual model that will serve requests.
fn resolve_auth_compatible_model(
    provider: &str,
    model: &str,
    client: &stencila_models3::client::Client,
) -> AgentResult<(String, bool, Option<String>)> {
    let auth_type = client.auth_type(provider);
    let actual_model = resolve_model_alias(model)
        .as_ref()
        .map_or_else(|| model.to_string(), |(_, concrete_id)| concrete_id.clone());

    if model_supports_auth_type(&actual_model, &auth_type) {
        return Ok((actual_model, false, None));
    }

    if let Some(fallback_model) =
        find_compatible_fallback_model(provider, &actual_model, &auth_type)
    {
        let fallback_reason = format!(
            "Model {actual_model} is not compatible with the current auth type; \
             using {fallback_model} instead"
        );
        return Ok((fallback_model, true, Some(fallback_reason)));
    }

    Err(AgentError::Sdk(
        stencila_models3::error::SdkError::Configuration {
            message: format!(
                "Model '{actual_model}' requires {auth_type:?} authentication, \
                 but the current credentials for '{provider}' use a different \
                 auth type and no compatible fallback model was found. \
                 Set {} to use this model.",
                api_key_env_hint(provider)
            ),
        },
    ))
}

/// Resolve a model alias to its concrete catalog ID.
///
/// Returns `Some((alias, concrete_id))` when the model name is an alias
/// that maps to a different concrete model ID in the catalog, or `None`
/// when the name is already a concrete ID (or not found in the catalog).
fn resolve_model_alias(model: &str) -> Option<(String, String)> {
    let info = stencila_models3::catalog::get_model_info(model).ok()??;
    if info.id != model {
        Some((model.to_string(), info.id))
    } else {
        None
    }
}

/// Detect the best available CLI provider by checking for installed binaries.
///
/// Checks for `claude`, `codex`, and `gemini` binaries on PATH.
/// Returns the first available CLI provider, or `None` if no supported
/// CLI binary is found.
fn detect_cli_provider() -> Option<String> {
    use std::process::Command;

    static CLI_BINARIES: &[(&str, &str)] = &[
        ("claude-cli", "claude"),
        ("codex-cli", "codex"),
        ("gemini-cli", "gemini"),
    ];

    for &(provider, binary) in CLI_BINARIES {
        let found = if cfg!(target_os = "windows") {
            Command::new("where").arg(binary).output()
        } else {
            Command::new("which").arg(binary).output()
        };
        if found.is_ok_and(|o| o.status.success()) {
            return Some(provider.to_string());
        }
    }

    None
}

/// Check if a model supports the given authentication type.
fn model_supports_auth_type(
    model_id: &str,
    auth_type: &stencila_models3::client::AuthType,
) -> bool {
    use stencila_models3::catalog;
    use stencila_models3::client::AuthType;

    // Unknown auth type — be permissive
    if *auth_type == AuthType::Unknown {
        return true;
    }

    let model_info = match catalog::get_model_info(model_id) {
        Ok(Some(info)) => info,
        _ => return true, // If we can't get model info, assume it's compatible
    };

    // If model doesn't specify auth_types, assume it's compatible with all
    if model_info.auth_types.is_empty() {
        return true;
    }

    model_info.auth_types.contains(auth_type)
}

/// Find a compatible fallback model for the given provider and auth type.
fn find_compatible_fallback_model(
    provider: &str,
    original_model: &str,
    auth_type: &stencila_models3::client::AuthType,
) -> Option<String> {
    use stencila_models3::catalog;

    let models = match catalog::list_models(Some(provider)) {
        Ok(models) => models,
        Err(_) => return None,
    };

    let family_prefix = model_family_prefix(original_model);

    // The catalog is already ordered best-first within each provider, so the
    // first compatible model in the filtered list is the preferred fallback.
    models
        .iter()
        .find(|m| {
            m.id != original_model
                && model_supports_auth_type(&m.id, auth_type)
                && family_prefix.is_none_or(|prefix| m.id.starts_with(prefix))
        })
        .map(|m| m.id.clone())
}

fn model_family_prefix(model_id: &str) -> Option<&'static str> {
    static PREFIXES: &[&str] = &["gpt-", "o1", "o3", "o4", "claude-", "gemini", "mistral"];
    PREFIXES
        .iter()
        .find(|&&prefix| model_id.starts_with(prefix))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an empty client with no providers.
    fn empty_client() -> stencila_models3::client::Client {
        stencila_models3::client::Client::builder()
            .build()
            .expect("should build empty client")
    }

    // -----------------------------------------------------------------------
    // route_direct tests (single provider + single model)
    // -----------------------------------------------------------------------

    #[test]
    fn explicit_cli_provider_routes_to_cli() {
        let client = empty_client();
        let decision = route_direct(Some("claude-cli"), Some("my-model"), &client)
            .expect("should route claude-cli");
        assert_eq!(
            decision.route,
            SessionRoute::Cli {
                provider: "claude-cli".into(),
                model: Some("my-model".into()),
            }
        );
    }

    #[test]
    fn explicit_cli_provider_no_model() {
        let client = empty_client();
        let decision =
            route_direct(Some("codex-cli"), None, &client).expect("should route codex-cli");
        assert_eq!(
            decision.route,
            SessionRoute::Cli {
                provider: "codex-cli".into(),
                model: None,
            }
        );
    }

    #[test]
    fn explicit_provider_and_model_no_auth_falls_back_to_cli() {
        let client = empty_client();
        let decision = route_direct(Some("anthropic"), Some("claude-sonnet-4-20250514"), &client)
            .expect("should route anthropic with model");
        // No API auth → falls back to claude-cli
        assert_eq!(
            decision.route,
            SessionRoute::Cli {
                provider: "claude-cli".into(),
                model: Some("claude-sonnet-4-20250514".into()),
            }
        );
    }

    #[test]
    fn explicit_provider_no_model_uses_default_alias() {
        let client = empty_client();
        let decision = route_direct(Some("anthropic"), None, &client)
            .expect("should route anthropic without model");
        assert_eq!(
            decision.route,
            SessionRoute::Cli {
                provider: "claude-cli".into(),
                model: Some("claude".into()),
            }
        );
    }

    #[test]
    fn mistral_provider_without_model_uses_default() {
        let client = empty_client();
        let result = route_direct(Some("mistral"), None, &client);
        // No API auth and no CLI mapping → error asking for API key
        let msg = result
            .expect_err("should error without API key")
            .to_string();
        assert!(msg.contains("MISTRAL_API_KEY"));
    }

    #[test]
    fn mistral_provider_with_model_no_auth_errors_with_hint() {
        let client = empty_client();
        let result = route_direct(Some("mistral"), Some("mistral-large"), &client);
        let msg = result
            .expect_err("should error without API key")
            .to_string();
        assert!(msg.contains("MISTRAL_API_KEY"));
    }

    #[test]
    fn provider_without_default_model_errors() {
        let client = empty_client();
        let result = route_direct(Some("unknown-provider"), None, &client);
        assert!(result.is_err());
    }

    #[test]
    fn no_provider_no_model_with_no_cli_errors() {
        let client = empty_client();
        let result = route_direct(None, None, &client);
        // When no API providers and no CLI binaries on PATH, should either
        // fall back to a detected CLI or return an actionable error.
        match result {
            Ok(decision) => match decision.route {
                SessionRoute::Cli { model, .. } => assert_eq!(model, None),
                SessionRoute::Api { .. } => panic!("Expected CLI route or error"),
            },
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("No API providers configured"));
            }
        }
    }

    // -----------------------------------------------------------------------
    // route_direct explained tests
    // -----------------------------------------------------------------------

    #[test]
    fn explained_explicit_cli_has_correct_metadata() {
        let client = empty_client();
        let decision = route_direct(Some("claude-cli"), Some("my-model"), &client)
            .expect("should explain claude-cli route");

        assert_eq!(decision.provider_source, ProviderSource::AgentExplicit);
        assert_eq!(decision.model_source, ModelSource::AgentExplicit);
        assert!(!decision.fallback_used);
        assert!(decision.fallback_reason.is_none());
    }

    #[test]
    fn explained_cli_no_model_uses_cli_default() {
        let client = empty_client();
        let decision = route_direct(Some("gemini-cli"), None, &client)
            .expect("should explain gemini-cli route");

        assert_eq!(decision.provider_source, ProviderSource::AgentExplicit);
        assert_eq!(decision.model_source, ModelSource::CliDefault);
    }

    #[test]
    fn explained_api_fallback_has_warning() {
        let client = empty_client();
        let decision = route_direct(Some("anthropic"), Some("claude-sonnet-4-20250514"), &client)
            .expect("should explain anthropic fallback route");

        assert!(decision.fallback_used);
        assert!(decision.fallback_reason.is_some());
        assert!(decision.fallback_warning().is_some());
        let warning = decision
            .fallback_warning()
            .expect("should have fallback warning");
        assert!(warning.contains("ANTHROPIC_API_KEY"));
        assert!(warning.starts_with("Falling back to claude-cli"));
    }

    #[test]
    fn explained_default_alias_model_source() {
        let client = empty_client();
        let decision =
            route_direct(Some("openai"), None, &client).expect("should explain openai route");

        assert_eq!(decision.model_source, ModelSource::DefaultAlias);
        // Model should be "gpt" (the default alias for openai)
        match &decision.route {
            SessionRoute::Cli { model, .. } => {
                assert_eq!(model.as_deref(), Some("gpt"));
            }
            SessionRoute::Api { model, .. } => {
                assert_eq!(model, "gpt");
            }
        }
    }

    #[test]
    fn explained_no_providers_cli_default_or_error() {
        let client = empty_client();
        let result = route_direct(None, None, &client);
        // When no API providers and no CLI binaries on PATH, should either
        // fall back to a detected CLI or return an actionable error.
        match result {
            Ok(decision) => {
                assert_eq!(decision.provider_source, ProviderSource::CliDefault);
                assert_eq!(decision.model_source, ModelSource::CliDefault);
                assert!(!decision.fallback_used);
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("No API providers configured"));
            }
        }
    }

    #[test]
    fn summary_format_includes_provider_and_backend() {
        let client = empty_client();
        let decision = route_direct(Some("claude-cli"), Some("my-model"), &client)
            .expect("should explain claude-cli for summary");

        let summary = decision.summary();
        assert!(summary.contains("claude-cli"));
        assert!(summary.contains("CLI"));
        assert!(summary.contains("my-model"));
    }

    // -----------------------------------------------------------------------
    // helper function tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_cli_provider_returns_true_for_known() {
        assert!(is_cli_provider("claude-cli"));
        assert!(is_cli_provider("codex-cli"));
        assert!(is_cli_provider("gemini-cli"));
    }

    #[test]
    fn is_cli_provider_returns_false_for_api() {
        assert!(!is_cli_provider("anthropic"));
        assert!(!is_cli_provider("openai"));
    }

    #[test]
    fn api_to_cli_mapping() {
        assert_eq!(api_to_cli("anthropic"), Some("claude-cli"));
        assert_eq!(api_to_cli("openai"), Some("codex-cli"));
        assert_eq!(api_to_cli("gemini"), Some("gemini-cli"));
        assert_eq!(api_to_cli("google"), Some("gemini-cli"));
        assert_eq!(api_to_cli("mistral"), None);
    }

    #[test]
    fn cli_to_api_mapping() {
        assert_eq!(cli_to_api("claude-cli"), Some("anthropic"));
        assert_eq!(cli_to_api("codex-cli"), Some("openai"));
        assert_eq!(cli_to_api("gemini-cli"), Some("gemini"));
        assert_eq!(cli_to_api("unknown"), None);
    }

    #[test]
    fn default_model_for_known_providers() {
        assert_eq!(default_model("anthropic"), Some("claude"));
        assert_eq!(default_model("openai"), Some("gpt"));
        assert_eq!(default_model("gemini"), Some("gemini"));
        assert_eq!(default_model("google"), Some("gemini"));
        assert_eq!(default_model("mistral"), Some("mistral-large-latest"));
        assert_eq!(default_model("unknown"), None);
    }
}
