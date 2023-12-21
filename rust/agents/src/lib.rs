use std::sync::Arc;

use agent::{
    common::{
        eyre::{bail, Result},
        tracing,
    },
    Agent, AgentIO, GenerateDetails, GenerateOptions, GenerateTask,
};

pub use agent;

pub mod testing;
mod testing_db;

/// Get a list of available agents in descending preference rank
pub async fn list() -> Vec<Arc<dyn Agent>> {
    let mut all = Vec::new();

    match agent_custom::list().await {
        Ok(mut agents) => all.append(&mut agents),
        Err(error) => tracing::error!("While listing Stencila agents: {error}"),
    }

    match agent_openai::list().await {
        Ok(mut agents) => all.append(&mut agents),
        Err(error) => tracing::debug!("While listing OpenAI agents: {error}"),
    }

    match agent_anthropic::list().await {
        Ok(mut agents) => all.append(&mut agents),
        Err(error) => tracing::debug!("While listing Anthropic agents: {error}"),
    }

    match agent_ollama::list().await {
        Ok(mut agents) => all.append(&mut agents),
        Err(error) => tracing::debug!("While listing Ollama agents: {error}"),
    }

    all.sort_by(|a, b| a.preference_rank().cmp(&b.preference_rank()).reverse());

    all
}

/// Perform a `GenerateTask` and return the generated content as a string
pub async fn generate_content(
    task: GenerateTask,
    options: &GenerateOptions,
) -> Result<(String, GenerateDetails)> {
    let agents = list().await;
    for agent in agents {
        if agent.supports_task(&task) && agent.supports_from_to(AgentIO::Text, AgentIO::Text) {
            return agent.text_to_text(task, options).await;
        }
    }

    bail!("Unable to delegate the task, no agents with suitable capabilities")
}
