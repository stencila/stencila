use std::sync::Arc;

use agent::{
    common::{
        eyre::{bail, Result},
        itertools::Itertools,
        tracing,
    },
    Agent, AgentIO, GenerateDetails, GenerateOptions, GenerateTask, GenerateOutput,
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
pub async fn perform_task(
    task: GenerateTask,
    options: &GenerateOptions,
) -> Result<(GenerateOutput, GenerateDetails)> {
    let agents = list().await;

    // It is tempting to use the position_max iterator method here but, in the case of
    // ties, that returns the agent with the higher index (ie. lower preference), whereas
    // we want the one with the lowest index.
    let mut best = (0., 0);
    for (index, agent) in agents.iter().enumerate() {
        let score = agent.suitability_score(&task);
        if score > best.0 {
            best = (score, index);
        }
    }

    let (max, index) = best;

    if max == 0. {
        bail!("Unable to delegate the task, no agents with suitable capabilities")
    }

    let Some(agent) = agents.get(index) else {
        bail!("Best agent not in list of agents!?")
    };

    agent.perform_task(task, options).await
}
