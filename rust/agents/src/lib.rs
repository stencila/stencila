use std::sync::Arc;

use cached::proc_macro::cached;

use agent::{
    common::{
        eyre::{bail, Result},
        tracing,
    },
    Agent, AgentIO, GenerateOptions,
};

pub use agent;

/// Get a list of available agents
///
/// Memoizes the result for an hour to reduce the number of times that
/// remote APIs need to be called to get a list of available models.
#[cached(time = 3600)]
pub async fn list() -> Vec<Arc<dyn Agent>> {
    let mut list = Vec::new();

    // The order that agents are added matters because the first
    // agent capable of executing an instruction will be used (unless a
    // specific model is specified). Generally, "better" agents should
    // come first.

    match agent_custom::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing custom agents: {error}"),
    }

    match agent_openai::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing OpenAI agents: {error}"),
    }

    match agent_ollama::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing Ollama agents: {error}"),
    }

    list
}

/// Generate text
///
/// Returns a tuple of the agent that did the generation and
/// the string it generated
pub async fn text_to_text(
    instruction: &str,
    agent_name: &Option<String>,
    options: &GenerateOptions,
) -> Result<(Arc<dyn Agent>, String)> {
    for agent in list().await {
        let should_use = if let Some(agent_name) = &agent_name {
            &agent.name() == agent_name
        } else {
            agent.supports_from_to(AgentIO::Text, AgentIO::Text)
        };

        if should_use {
            return Ok((
                agent.clone(),
                agent.text_to_text(instruction, options).await?,
            ));
        }
    }

    bail!("None of the available agents support text generation or the specified agent is not available")
}

/// Generate image
///
/// Returns a tuple of the agent that did the generation and
/// the string it generated
pub async fn text_to_image(
    instruction: &str,
    agent_name: Option<String>,
    options: &GenerateOptions,
) -> Result<(Arc<dyn Agent>, String)> {
    for agent in list().await {
        let should_use = if let Some(agent_name) = &agent_name {
            &agent.name() == agent_name
        } else {
            agent.supports_from_to(AgentIO::Text, AgentIO::Image)
        };

        if should_use {
            return Ok((
                agent.clone(),
                agent.text_to_image(instruction, options).await?,
            ));
        }
    }

    bail!("None of the available agents support image generation or the specified agent is not available")
}
