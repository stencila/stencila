//! High-level convenience functions for callers that don't want to manage
//! agent discovery etc manually.

use std::sync::Arc;

use eyre::Result;

use crate::agent_def::{self, AgentInstance};
use crate::error::{AgentError, AgentResult};
use crate::events::EventReceiver;
use crate::execution::LocalExecutionEnvironment;
use crate::profiles::{AnthropicProfile, GeminiProfile, OpenAiProfile};
use crate::prompts;
use crate::session::{Models3Client, Session};
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

/// Default model for each provider when none is specified.
///
/// Uses the alias that points to the latest for each provider
fn default_model(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("claude"),
        "openai" => Some("gpt"),
        "gemini" | "google" => Some("gemini"),
        _ => None,
    }
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

/// Create an agent session from a named agent definition.
///
/// Discovers the agent by name (searching workspace then user config),
/// reads its instructions from the AGENT.md body, builds a [`SessionConfig`]
/// from its schema fields, and creates a session.
///
/// If `name` is `"default"`, it is resolved to the agent configured in
/// `[agents].default` in `stencila.toml` (falling back to `"default"` if unset).
///
/// Returns the discovered [`AgentInstance`] alongside the session and event
/// receiver so callers can inspect agent metadata (name, description, etc.).
///
/// # Errors
///
/// Returns an error if the agent is not found, the AGENT.md cannot be read,
/// or session creation fails (no API keys, unsupported provider, etc.).
pub async fn create_session(name: &str) -> AgentResult<(AgentInstance, Session, EventReceiver)> {
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

    let client = stencila_models3::client::Client::from_env().map_err(AgentError::Sdk)?;

    let provider_name = match &agent.provider {
        Some(p) => p.to_string(),
        None => match client.select_provider() {
            Some(p) => p.to_string(),
            None => {
                return Err(AgentError::Sdk(
                    stencila_models3::error::SdkError::Configuration {
                        message: "No API keys found. \
                                  Set ANTHROPIC_API_KEY, OPENAI_API_KEY, or GEMINI_API_KEY."
                            .to_string(),
                    },
                ));
            }
        },
    };

    let model_name = match &agent.model {
        Some(m) => m.to_string(),
        None => match default_model(&provider_name) {
            Some(m) => m.to_string(),
            None => {
                return Err(AgentError::Sdk(
                    stencila_models3::error::SdkError::Configuration {
                        message: format!(
                            "No default model for provider '{provider_name}'. \
                             Please specify a model explicitly."
                        ),
                    },
                ));
            }
        },
    };

    config.commit_instructions = Some(prompts::build_commit_instructions());

    let max_timeout = config.max_command_timeout_ms;

    let mut profile: Box<dyn crate::profile::ProviderProfile> = match provider_name.as_str() {
        "anthropic" => Box::new(AnthropicProfile::new(&model_name, max_timeout)?),
        "openai" => Box::new(OpenAiProfile::new(&model_name, max_timeout)?),
        "gemini" | "google" => Box::new(GeminiProfile::new(&model_name, max_timeout)?),
        name => {
            return Err(AgentError::Sdk(
                stencila_models3::error::SdkError::Configuration {
                    message: format!(
                        "Provider '{name}' is not supported. \
                         Set ANTHROPIC_API_KEY, OPENAI_API_KEY, or GEMINI_API_KEY."
                    ),
                },
            ));
        }
    };

    let env = Arc::new(LocalExecutionEnvironment::new("."));
    let llm_client = Arc::new(Models3Client::new(client));

    let (system_prompt, mcp_context) =
        prompts::build_system_prompt(&mut *profile, &*env, &config).await?;

    let (session, event_receiver) = Session::new(
        profile,
        env,
        llm_client,
        config,
        system_prompt,
        0,
        mcp_context,
    );

    Ok((agent, session, event_receiver))
}
