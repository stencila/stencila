use agent::{
    common::{
        eyre::{bail, Result},
        tracing,
    },
    Agent, AgentIO,
};

pub use agent;

/// Get a list of available agents
pub async fn list() -> Vec<Box<dyn Agent>> {
    let mut list = Vec::new();

    match agents_ollama::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing Ollama agents: {error}"),
    }

    match agents_openai::list().await {
        Ok(mut agents) => list.append(&mut agents),
        Err(error) => tracing::debug!("While listing OpenAI agents: {error}"),
    }

    list
}

/// Generate text
pub async fn generate_text(instruction: &str, agent: Option<String>) -> Result<(String, String)> {
    let agent_name = agent.unwrap_or_default();
    for agent in list().await {
        if agent.name() == agent_name || agent.supports_generating(AgentIO::Text) {
            return Ok((agent.name(), agent.generate_text(instruction, None).await?));
        }
    }

    bail!("None of the available agents support text generation or the specified agent is not available")
}

/// Generate image
pub async fn generate_image(instruction: &str, agent: Option<String>) -> Result<(String, String)> {
    let agent_name = agent.unwrap_or_default();
    for agent in list().await {
        if agent.name() == agent_name || agent.supports_generating(AgentIO::Image) {
            return Ok((agent.name(), agent.generate_image(instruction, None).await?));
        }
    }

    bail!("None of the available agents support image generation or the specified agent is not available")
}
