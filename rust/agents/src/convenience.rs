//! High-level session factory for callers that don't want to manage
//! agent discovery etc manually.

use std::sync::Arc;

use crate::agent_def::{self, AgentInstance};
use crate::error::{AgentError, AgentResult};
use crate::events::EventReceiver;
use crate::execution::LocalExecutionEnvironment;
use crate::profiles::{AnthropicProfile, GeminiProfile, OpenAiProfile};
use crate::prompts;
use crate::session::{Models3Client, Session};
use crate::types::SessionConfig;

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

/// Create an agent session from a named agent definition.
///
/// Discovers the agent by name (searching workspace then user config),
/// reads its instructions from the AGENT.md body, builds a [`SessionConfig`]
/// from its schema fields, and creates a session.
///
/// Returns the discovered [`AgentInstance`] alongside the session and event
/// receiver so callers can inspect agent metadata (name, description, etc.).
///
/// # Errors
///
/// Returns an error if the agent is not found, the AGENT.md cannot be read,
/// or session creation fails (no API keys, unsupported provider, etc.).
pub async fn create_session(name: &str) -> AgentResult<(AgentInstance, Session, EventReceiver)> {
    let cwd = std::env::current_dir().map_err(|e| {
        AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
            message: format!("Failed to get current directory: {e}"),
        })
    })?;

    let agent = agent_def::get_by_name(&cwd, name).await.map_err(|e| {
        AgentError::Sdk(stencila_models3::error::SdkError::Configuration {
            message: format!("Agent not found: {e}"),
        })
    })?;

    let config = SessionConfig::from_agent(&agent).await.map_err(|e| {
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
