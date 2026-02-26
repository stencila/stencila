//! High-level convenience functions for callers that don't want to manage
//! agent discovery etc manually.

use std::sync::Arc;

use eyre::Result;

use crate::agent_def::{self, AgentInstance};
use crate::agent_session::AgentSession;
use crate::api_session::{ApiSession, Models3Client};
use crate::error::{AgentError, AgentResult};
use crate::events::EventReceiver;
use crate::execution::LocalExecutionEnvironment;
use crate::profiles::{AnthropicProfile, GeminiProfile, OpenAiProfile};
use crate::prompts;
use crate::routing::{self, SessionRoute};
use crate::types::SessionConfig;

/// Options for [`create_agent`].
#[derive(Default)]
pub struct CreateAgentOptions<'a> {
    /// Create in user config directory instead of workspace.
    pub user: bool,

    /// Model identifier (e.g. "claude-sonnet-4-5").
    pub model: Option<&'a str>,

    /// Provider name (e.g. "anthropic", "openai").
    pub provider: Option<&'a str>,

    /// Instructions (becomes the Markdown body of AGENT.md).
    pub instructions: Option<&'a str>,
}

/// Create a new agent definition on disk.
///
/// Creates a new agent directory with an `AGENT.md` file containing the given
/// name and description as YAML frontmatter. Optionally includes model,
/// provider, and instructions (Markdown body).
///
/// By default creates in the workspace's `.stencila/agents/` directory; set
/// [`CreateAgentOptions::user`] to `true` to create in the user config
/// directory (`~/.config/stencila/agents/`) instead.
///
/// Returns the loaded [`AgentInstance`] for the newly created agent.
///
/// # Errors
///
/// Returns an error if the name fails validation, the agent already exists,
/// or file I/O fails.
pub async fn create_agent(
    name: &str,
    description: &str,
    options: &CreateAgentOptions<'_>,
) -> Result<AgentInstance> {
    let name_errors = crate::agent_validate::validate_name(name);
    if !name_errors.is_empty() {
        let msgs: Vec<String> = name_errors.iter().map(|e| e.to_string()).collect();
        eyre::bail!("Invalid agent name `{name}`: {}", msgs.join("; "));
    }

    let cwd = std::env::current_dir()?;

    let agents_dir = if options.user {
        stencila_dirs::get_app_dir(stencila_dirs::DirType::Agents, true)?
    } else {
        agent_def::closest_agents_dir(&cwd, true).await?
    };

    let agent_dir = agents_dir.join(name);

    if agent_dir.exists() {
        eyre::bail!("Agent `{name}` already exists at `{}`", agent_dir.display());
    }

    tokio::fs::create_dir_all(&agent_dir).await?;

    // Build YAML frontmatter using proper serialization to handle special characters
    let mut frontmatter_map = serde_yaml::Mapping::new();
    frontmatter_map.insert("name".into(), name.into());
    frontmatter_map.insert("description".into(), description.into());
    if let Some(model) = options.model {
        frontmatter_map.insert("model".into(), model.into());
    }
    if let Some(provider) = options.provider {
        frontmatter_map.insert("provider".into(), provider.into());
    }
    let frontmatter = serde_yaml::to_string(&frontmatter_map)?;
    // serde_yaml adds a trailing newline; trim it since the template adds its own
    let frontmatter = frontmatter.trim_end();

    let body = options
        .instructions
        .filter(|s| !s.is_empty())
        .unwrap_or("TODO: Add instructions for this agent.");

    let content = format!("---\n{frontmatter}\n---\n\n{body}\n");

    let agent_md = agent_dir.join("AGENT.md");
    tokio::fs::write(&agent_md, content).await?;

    agent_def::get_by_name(&cwd, name).await
}

/// If `name` is `"default"`, resolve it to the configured default agent name
/// from `[agents].default` in `stencila.toml`. Returns the name unchanged
/// if it is not `"default"` or if no config is set.
pub fn resolve_default_agent_name(name: &str) -> String {
    if name != "default" {
        return name.to_string();
    }
    stencila_config::get()
        .ok()
        .and_then(|config| config.agents.as_ref()?.default.clone())
        .unwrap_or_else(|| name.to_string())
}

/// Resolve commit attribution policy from configuration.
fn resolve_commit_attribution() -> stencila_config::CommitAttribution {
    stencila_config::get()
        .ok()
        .and_then(|config| config.agents.as_ref().and_then(|agents| agents.commit_attribution))
        .unwrap_or_default()
}

/// Shared preamble: resolve name, load agent definition, build session config.
async fn load_agent_and_config(name: &str) -> AgentResult<(AgentInstance, SessionConfig)> {
    let resolved_name = resolve_default_agent_name(name);

    let cwd = std::env::current_dir().map_err(|e| {
        AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
            message: format!("Failed to get current directory: {e}"),
        })
    })?;

    let agent = agent_def::get_by_name(&cwd, &resolved_name)
        .await
        .map_err(|e| {
            AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                message: format!("Agent not found: {e}"),
            })
        })?;

    let mut config = SessionConfig::from_agent(&agent).await.map_err(|e| {
        AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
            message: format!("Failed to build session config from agent: {e}"),
        })
    })?;

    let commit_attribution = resolve_commit_attribution();
    config.commit_instructions = Some(prompts::build_commit_instructions(commit_attribution));

    Ok((agent, config))
}

/// Create an agent session from a named agent definition.
///
/// Discovers the agent by name (searching workspace then user config),
/// reads its instructions from the AGENT.md body, builds a [`SessionConfig`]
/// from its schema fields, and routes the session:
///
/// 1. If the agent's provider is an explicit CLI provider (`claude-cli`,
///    `codex-cli`, `gemini-cli`), a CLI-backed session is always used.
/// 2. Otherwise an API provider is resolved (explicit, inferred from the model
///    name, or the default configured provider).
/// 3. If API credentials exist for that provider, an API session is created.
/// 4. If no API credentials exist but a corresponding CLI tool is available
///    (e.g. `anthropic` → `claude-cli`), falls back to a CLI session.
/// 5. If no CLI mapping exists (e.g. `mistral`, `deepseek`), returns an error
///    asking the user to set the appropriate API key.
///
/// If `name` is `"default"`, it is resolved to the agent configured in
/// `[agents].default` in `stencila.toml` (falling back to `"default"` if unset).
///
/// Returns the discovered [`AgentInstance`] alongside the unified
/// [`AgentSession`] and event receiver so callers can inspect agent metadata
/// (name, description, etc.).
///
/// # Errors
///
/// Returns an error if the agent is not found, the AGENT.md cannot be read,
/// or session creation fails (no API keys, unsupported provider, etc.).
pub async fn create_session(
    name: &str,
) -> AgentResult<(AgentInstance, AgentSession, EventReceiver)> {
    let (agent, config) = load_agent_and_config(name).await?;

    let client = stencila_models3::client::Client::from_env().ok();

    // Build an empty client as fallback for routing decisions when no API
    // keys are available (from_env returned Err).
    let empty_client;
    let client_ref = match client.as_ref() {
        Some(c) => c,
        None => {
            empty_client = stencila_models3::client::Client::builder()
                .build()
                .map_err(AgentError::Sdk)?;
            &empty_client
        }
    };

    let route = routing::route_session(
        agent.provider.as_deref(),
        agent.model.as_deref(),
        client_ref,
    )?;

    match route {
        SessionRoute::Cli { provider, model } => {
            let (session, event_receiver) =
                create_cli_session_inner(&provider, model.as_deref(), &config)?;
            Ok((agent, AgentSession::Cli(session), event_receiver))
        }
        SessionRoute::Api { provider, model } => {
            let (session, event_receiver) = create_api_session_inner(
                &provider,
                &model,
                client.ok_or_else(|| {
                    AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
                        message: "No API client available".to_string(),
                    })
                })?,
                config,
            )
            .await?;
            Ok((agent, AgentSession::Api(session), event_receiver))
        }
    }
}

/// Build an API-backed [`Session`] from an already-resolved provider, model,
/// and authenticated client.
async fn create_api_session_inner(
    provider_name: &str,
    model_name: &str,
    client: stencila_models3::client::Client,
    config: SessionConfig,
) -> AgentResult<(ApiSession, EventReceiver)> {
    let max_timeout = config.max_command_timeout_ms;

    let mut profile: Box<dyn crate::profile::ProviderProfile> = match provider_name {
        "anthropic" => Box::new(AnthropicProfile::new(model_name, max_timeout)?),
        "openai" => Box::new(OpenAiProfile::new(model_name, max_timeout)?),
        "gemini" | "google" => Box::new(GeminiProfile::new(model_name, max_timeout)?),
        name => {
            return Err(AgentError::Sdk(
                stencila_models3::error::SdkError::Configuration {
                    message: format!(
                        "Provider '{name}' is not yet supported by the agents layer. \
                         Supported providers: anthropic, openai, gemini."
                    ),
                },
            ));
        }
    };

    let env = Arc::new(LocalExecutionEnvironment::new("."));
    let llm_client = Arc::new(Models3Client::new(client));

    let (system_prompt, mcp_context) =
        prompts::build_system_prompt(&mut *profile, &*env, &config).await?;

    let (session, event_receiver) = ApiSession::new(
        profile,
        env,
        llm_client,
        config,
        system_prompt,
        0,
        mcp_context,
    );

    Ok((session, event_receiver))
}

/// Build a CLI-backed [`CliSession`] from a resolved CLI provider name,
/// optional model override, and session config.
///
/// The `cli_provider` must be one of `claude-cli`, `codex-cli`, or
/// `gemini-cli`. When a `model` is provided (e.g. from API→CLI fallback),
/// it is forwarded to the CLI tool; otherwise the CLI uses the model from
/// the session config (which may itself be `None`, letting the CLI use its
/// own default).
fn create_cli_session_inner(
    cli_provider: &str,
    model: Option<&str>,
    config: &SessionConfig,
) -> AgentResult<(crate::cli_providers::CliSession, EventReceiver)> {
    use crate::cli_providers::{CliProviderConfig, CliSession};

    let cli_config = CliProviderConfig::from_session_config(config, model);

    let provider: Box<dyn crate::cli_providers::CliProvider> = match cli_provider {
        "claude-cli" => Box::new(crate::cli_providers::claude::ClaudeCliProvider::new(
            cli_config,
        )),
        "codex-cli" => Box::new(crate::cli_providers::codex::CodexCliProvider::new(
            cli_config,
        )),
        "gemini-cli" => Box::new(crate::cli_providers::gemini::GeminiCliProvider::new(
            cli_config,
        )),
        other => {
            return Err(AgentError::CliUnsupported {
                operation: format!(
                    "Provider '{other}' is not a CLI provider. \
                     Use claude-cli, codex-cli, or gemini-cli."
                ),
            });
        }
    };

    let (session, event_receiver) = CliSession::new(provider, config.clone());

    Ok((session, event_receiver))
}

/// Run a single prompt against a named agent and return the collected response
/// text.
///
/// Creates a session from the agent definition, submits the prompt, drains all
/// events to collect `AssistantTextDelta` tokens, and returns the assembled
/// response string. The session is closed on drop.
///
/// This is a fire-and-forget convenience for callers that don't need streaming.
///
/// # Errors
///
/// Returns an error if the agent is not found, session creation fails, or the
/// LLM call fails.
pub async fn run_prompt(name: &str, prompt: &str) -> AgentResult<String> {
    let (_agent, mut session, mut event_rx) = create_session(name).await?;

    let mut submit_future = Box::pin(session.submit(prompt));
    let mut submit_done = false;
    let mut submit_result: Option<AgentResult<()>> = None;
    let mut collected_text = String::new();

    loop {
        tokio::select! {
            biased;

            event = event_rx.recv() => {
                let Some(event) = event else {
                    break;
                };
                if event.kind == crate::types::EventKind::AssistantTextDelta
                    && let Some(serde_json::Value::String(delta)) = event.data.get("delta") {
                        collected_text.push_str(delta);
                    }
            }

            result = &mut submit_future, if !submit_done => {
                submit_done = true;
                submit_result = Some(result);
            }
        }

        if submit_done {
            while let Ok(event) = event_rx.try_recv() {
                if event.kind == crate::types::EventKind::AssistantTextDelta
                    && let Some(serde_json::Value::String(delta)) = event.data.get("delta")
                {
                    collected_text.push_str(delta);
                }
            }
            break;
        }
    }

    if let Some(Err(e)) = submit_result {
        return Err(e);
    }

    Ok(collected_text)
}
