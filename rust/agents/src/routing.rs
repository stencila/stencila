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
fn default_model(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("claude"),
        "openai" => Some("gpt"),
        "gemini" | "google" => Some("gemini"),
        _ => None,
    }
}

/// Hint for which environment variable to set for a given provider.
fn api_key_env_hint(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "gemini" | "google" => "GEMINI_API_KEY",
        "mistral" => "MISTRAL_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
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
    /// Selected as the first available configured API provider.
    DefaultConfig,
    /// Fell back to a CLI tool because no API providers were available.
    CliDefault,
}

impl fmt::Display for ProviderSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentExplicit => write!(f, "from agent definition"),
            Self::InferredFromModel => write!(f, "inferred from model"),
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
    /// Default alias for the resolved provider.
    DefaultAlias,
    /// CLI tool's own default (no model passed).
    CliDefault,
}

impl fmt::Display for ModelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentExplicit => write!(f, "from agent definition"),
            Self::DefaultAlias => write!(f, "default alias for provider"),
            Self::CliDefault => write!(f, "CLI default"),
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
}

impl RoutingDecision {
    /// Format as a concise one-line summary for user display.
    pub fn summary(&self) -> String {
        let (backend, provider, model) = match &self.route {
            SessionRoute::Api { provider, model } => ("API", provider.as_str(), Some(model.as_str())),
            SessionRoute::Cli { provider, model } => {
                ("CLI", provider.as_str(), model.as_deref())
            }
        };

        let model_display = if let Some((alias, concrete)) = &self.alias_resolution {
            format!("{alias} → {concrete}")
        } else if let Some(m) = model {
            m.to_string()
        } else {
            "default".to_string()
        };

        format!(
            "Using {provider} / {model_display} ({backend}; {provider_source})",
            provider_source = self.provider_source,
        )
    }

    /// Format the fallback warning, if a fallback occurred.
    pub fn fallback_warning(&self) -> Option<String> {
        if !self.fallback_used {
            return None;
        }
        let reason = self.fallback_reason.as_deref().unwrap_or("no API credentials");
        let SessionRoute::Cli { provider, .. } = &self.route else {
            return None;
        };
        Some(format!(
            "Falling back to {provider} — {reason}"
        ))
    }
}

/// Decide whether to use an API or CLI backend for the given agent.
///
/// The decision follows these rules in order:
///
/// 1. **Explicit CLI provider** — if the agent's `provider` is a `*-cli` name,
///    always route to CLI.
/// 2. **Resolve an API provider** for the model (explicit, inferred, or default).
/// 3. **API auth available** — if the `models3::Client` has credentials for
///    that provider, route to API.
/// 4. **No auth, mapped CLI exists** — fall back to the corresponding CLI tool
///    (e.g. `anthropic` → `claude-cli`).
/// 5. **No auth, no CLI mapping** — return an error.
///
/// This is a thin wrapper around [`route_session_explained`] that discards
/// the explanation metadata.
pub fn route_session(
    agent_provider: Option<&str>,
    agent_model: Option<&str>,
    client: &stencila_models3::client::Client,
) -> AgentResult<SessionRoute> {
    Ok(route_session_explained(agent_provider, agent_model, client)?.route)
}

/// Like [`route_session`] but returns a full [`RoutingDecision`] with
/// explanation metadata for UX surfaces.
pub fn route_session_explained(
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
        let alias_resolution = agent_model.and_then(resolve_alias_preview);
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
        });
    }

    // 2. Resolve API provider + model
    let (api_provider, model, provider_source, model_source) =
        match (agent_provider, agent_model) {
            (Some(p), Some(m)) => (
                p.to_string(),
                m.to_string(),
                ProviderSource::AgentExplicit,
                ModelSource::AgentExplicit,
            ),
            (Some(p), None) => {
                let m = default_model(p)
                    .ok_or_else(|| {
                        AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                            message: format!(
                                "No default model for provider '{p}'. \
                                 Please specify a model explicitly."
                            ),
                        })
                    })?
                    .to_string();
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
                    let m = default_model(p)
                        .ok_or_else(|| {
                            AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                                message: format!(
                                    "No default model for provider '{p}'. \
                                     Please specify a model explicitly."
                                ),
                            })
                        })?
                        .to_string();
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
                    });
                }
            }
        };

    let alias_resolution = resolve_alias_preview(&model);

    // 3. API auth available — use API
    if client.has_provider(&api_provider) {
        return Ok(RoutingDecision {
            route: SessionRoute::Api {
                provider: api_provider,
                model,
            },
            provider_source,
            model_source,
            alias_resolution,
            fallback_used: false,
            fallback_reason: None,
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
            fallback_reason: Some(format!(
                "No API key for {api_provider} (set {env_hint})"
            )),
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

/// Preview alias resolution without mutating any request.
///
/// Returns `Some((alias, concrete_id))` when the model name is an alias
/// that maps to a different concrete model ID in the catalog.
fn resolve_alias_preview(model: &str) -> Option<(String, String)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an empty client with no providers.
    fn empty_client() -> stencila_models3::client::Client {
        stencila_models3::client::Client::builder()
            .build()
            .unwrap()
    }

    // -----------------------------------------------------------------------
    // route_session tests
    // -----------------------------------------------------------------------

    #[test]
    fn explicit_cli_provider_routes_to_cli() {
        let client = empty_client();
        let route = route_session(Some("claude-cli"), Some("my-model"), &client).unwrap();
        assert_eq!(
            route,
            SessionRoute::Cli {
                provider: "claude-cli".into(),
                model: Some("my-model".into()),
            }
        );
    }

    #[test]
    fn explicit_cli_provider_no_model() {
        let client = empty_client();
        let route = route_session(Some("codex-cli"), None, &client).unwrap();
        assert_eq!(
            route,
            SessionRoute::Cli {
                provider: "codex-cli".into(),
                model: None,
            }
        );
    }

    #[test]
    fn explicit_provider_and_model_no_auth_falls_back_to_cli() {
        let client = empty_client();
        let route = route_session(Some("anthropic"), Some("claude-sonnet-4-20250514"), &client).unwrap();
        // No API auth → falls back to claude-cli
        assert_eq!(
            route,
            SessionRoute::Cli {
                provider: "claude-cli".into(),
                model: Some("claude-sonnet-4-20250514".into()),
            }
        );
    }

    #[test]
    fn explicit_provider_no_model_uses_default_alias() {
        let client = empty_client();
        let route = route_session(Some("anthropic"), None, &client).unwrap();
        assert_eq!(
            route,
            SessionRoute::Cli {
                provider: "claude-cli".into(),
                model: Some("claude".into()),
            }
        );
    }

    #[test]
    fn provider_without_default_model_errors() {
        let client = empty_client();
        let result = route_session(Some("mistral"), None, &client);
        assert!(result.is_err());
    }

    #[test]
    fn provider_without_cli_mapping_errors_when_no_auth() {
        let client = empty_client();
        let result = route_session(Some("mistral"), Some("mistral-large"), &client);
        assert!(result.is_err());
    }

    #[test]
    fn no_provider_no_model_with_no_cli_errors() {
        let client = empty_client();
        let result = route_session(None, None, &client);
        // When no API providers and no CLI binaries on PATH, should either
        // fall back to a detected CLI or return an actionable error.
        match result {
            Ok(SessionRoute::Cli { model, .. }) => assert_eq!(model, None),
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("No API providers configured"));
            }
            Ok(SessionRoute::Api { .. }) => panic!("Expected CLI route or error"),
        }
    }

    // -----------------------------------------------------------------------
    // route_session_explained tests
    // -----------------------------------------------------------------------

    #[test]
    fn explained_explicit_cli_has_correct_metadata() {
        let client = empty_client();
        let decision =
            route_session_explained(Some("claude-cli"), Some("my-model"), &client).unwrap();

        assert_eq!(decision.provider_source, ProviderSource::AgentExplicit);
        assert_eq!(decision.model_source, ModelSource::AgentExplicit);
        assert!(!decision.fallback_used);
        assert!(decision.fallback_reason.is_none());
    }

    #[test]
    fn explained_cli_no_model_uses_cli_default() {
        let client = empty_client();
        let decision = route_session_explained(Some("gemini-cli"), None, &client).unwrap();

        assert_eq!(decision.provider_source, ProviderSource::AgentExplicit);
        assert_eq!(decision.model_source, ModelSource::CliDefault);
    }

    #[test]
    fn explained_api_fallback_has_warning() {
        let client = empty_client();
        let decision = route_session_explained(
            Some("anthropic"),
            Some("claude-sonnet-4-20250514"),
            &client,
        )
        .unwrap();

        assert!(decision.fallback_used);
        assert!(decision.fallback_reason.is_some());
        assert!(decision.fallback_warning().is_some());
        let warning = decision.fallback_warning().unwrap();
        assert!(warning.contains("ANTHROPIC_API_KEY"));
        assert!(warning.starts_with("Falling back to claude-cli"));
    }

    #[test]
    fn explained_default_alias_model_source() {
        let client = empty_client();
        let decision =
            route_session_explained(Some("openai"), None, &client).unwrap();

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
        let result = route_session_explained(None, None, &client);
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
        let decision =
            route_session_explained(Some("claude-cli"), Some("my-model"), &client).unwrap();

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
        assert_eq!(default_model("mistral"), None);
    }
}
