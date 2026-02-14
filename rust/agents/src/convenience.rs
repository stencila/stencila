//! High-level session factory for callers that don't want to manage
//! model selection manually.
//!
//! [`create_session`] auto-detects available API keys via
//! `Client::from_env()`, selects a matching provider profile, builds the
//! system prompt, and returns a ready-to-use `(Session, EventReceiver)`.

use std::sync::Arc;

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

/// Create an agent session, optionally overriding the provider and model.
///
/// When `provider` is `None`, selects the first available provider based on
/// `models.providers` config order when set (otherwise registration order).
/// When `model` is `None`, uses the default model for the provider.
///
/// # Errors
///
/// Returns an error if no API keys are found, the provider is not one of
/// the three supported providers, or the provider has no default model
/// and `model` is `None`.
pub async fn create_session(
    provider: Option<&str>,
    model: Option<&str>,
) -> AgentResult<(Session, EventReceiver)> {
    create_session_with_instructions(provider, model, None).await
}

/// Create an agent session with optional user instructions.
///
/// Same as [`create_session`] but accepts an optional `user_instructions`
/// string that is injected into the session config as a per-session
/// system prompt override.
///
/// # Errors
///
/// Returns an error if no API keys are found, the provider is not one of
/// the three supported providers, or the provider has no default model
/// and `model` is `None`.
pub async fn create_session_with_instructions(
    provider: Option<&str>,
    model: Option<&str>,
    user_instructions: Option<String>,
) -> AgentResult<(Session, EventReceiver)> {
    let client = stencila_models3::client::Client::from_env().map_err(AgentError::Sdk)?;

    let provider_name = match provider {
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

    let model_name = match model {
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

    let config = SessionConfig {
        user_instructions,
        ..SessionConfig::default()
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

    Ok(Session::new(
        profile,
        env,
        llm_client,
        config,
        system_prompt,
        0,
        mcp_context,
    ))
}
