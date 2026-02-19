//! Session routing: decides whether an agent session should use an API
//! backend or fall back to a CLI tool.
//!
//! The core entry point is [`route_session`], which inspects the agent's
//! provider/model fields and the available API credentials to produce a
//! [`SessionRoute`].

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
pub fn route_session(
    agent_provider: Option<&str>,
    agent_model: Option<&str>,
    client: &stencila_models3::client::Client,
) -> AgentResult<SessionRoute> {
    // 1. Explicit CLI provider
    if let Some(p) = agent_provider
        && is_cli_provider(p)
    {
        return Ok(SessionRoute::Cli {
            provider: p.to_string(),
            model: agent_model.map(String::from),
        });
    }

    // 2. Resolve API provider + model
    let (api_provider, model) = match (agent_provider, agent_model) {
        (Some(p), Some(m)) => (p.to_string(), m.to_string()),
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
            (p.to_string(), m)
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
            (p, m.to_string())
        }
        (None, None) => {
            // Try to select a configured API provider first
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
                (p.to_string(), m)
            } else {
                // No API providers at all — fall back to the first CLI tool
                // (claude-cli) so users with CLI tools installed get a
                // working session without any API keys.
                return Ok(SessionRoute::Cli {
                    provider: CLI_PROVIDERS[0].to_string(),
                    model: None,
                });
            }
        }
    };

    // 3. API auth available — use API
    if client.has_provider(&api_provider) {
        return Ok(SessionRoute::Api {
            provider: api_provider,
            model,
        });
    }

    // 4. No auth — try mapped CLI fallback
    if let Some(cli) = api_to_cli(&api_provider) {
        return Ok(SessionRoute::Cli {
            provider: cli.to_string(),
            model: Some(model),
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
