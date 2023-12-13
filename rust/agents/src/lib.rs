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

    match agents_openai::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing OpenAI agents: {error}"),
    }

    match agents_ollama::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing Ollama agents: {error}"),
    }

    list
}

/// Generate text
pub async fn text_to_text(
    instruction: &str,
    agent_name: Option<String>,
    options: Option<GenerateOptions>,
) -> Result<(String, String)> {
    for agent in list().await {
        let should_use = if let Some(agent_name) = &agent_name {
            &agent.name() == agent_name
        } else {
            agent.supports_from_to(AgentIO::Text, AgentIO::Text)
        };

        if should_use {
            return Ok((
                agent.name(),
                agent.text_to_text(instruction, options).await?,
            ));
        }
    }

    bail!("None of the available agents support text generation or the specified agent is not available")
}

/// Generate image
pub async fn text_to_image(
    instruction: &str,
    agent_name: Option<String>,
) -> Result<(String, String)> {
    for agent in list().await {
        let should_use = if let Some(agent_name) = &agent_name {
            &agent.name() == agent_name
        } else {
            agent.supports_from_to(AgentIO::Text, AgentIO::Image)
        };

        if should_use {
            return Ok((agent.name(), agent.text_to_image(instruction, None).await?));
        }
    }

    bail!("None of the available agents support image generation or the specified agent is not available")
}
