use std::{env, sync::Arc};

use anthropic::{
    client::Client, config::AnthropicConfig, types::CompleteRequestBuilder, AI_PROMPT, HUMAN_PROMPT,
};

use agent::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        serde_json::json,
    },
    Agent, AgentIO, GenerateOptions,
};

/// An agent running on Anthropic
///
/// The environment variable ANTHROPIC_API_KEY must be set to use these agents.
pub struct AnthropicAgent {
    /// The name of the model including its version e.g. "claude-2.1"
    model: String,
}

impl AnthropicAgent {
    /// Create a Anthropic agent
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

#[async_trait]
impl Agent for AnthropicAgent {
    fn provider(&self) -> String {
        "anthropic".to_string()
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn supported_inputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    async fn text_to_text(&self, instruction: &str, options: &GenerateOptions) -> Result<String> {
        let (system_prompt, user_prompt) = self.render_prompt(
            &options.prompt_name,
            json!({
                "user_instruction": instruction
            }),
        )?;

        let cfg = AnthropicConfig::new()?;
        let client = Client::try_from(cfg)?;

        // Build completion request from `options`
        // TODO: We need to add the following which are not in the CompleteRequest (maybe by PR).
        // https://docs.anthropic.com/claude/reference/complete_post
        // temperature
        // top_k
        // top_p
        let complete_request = CompleteRequestBuilder::default()
            // The .._PROMPT values have embedded carriage returns.
            // System prompts in Claude are just put before the HUMAN_PROMPT.
            // https://docs.anthropic.com/claude/docs/how-to-use-system-prompts
            .prompt(format!(
                "{system_prompt}{HUMAN_PROMPT}{user_prompt}{AI_PROMPT}"
            ))
            .model(&self.model)
            // Not sure the best way to do this, but 256 is the default.
            .max_tokens_to_sample(options.max_tokens.unwrap_or(256) as usize)
            .stop_sequences(vec![HUMAN_PROMPT.to_string()])
            .build()?;

        // Send a completion request.
        let complete_response = client.complete(complete_request).await?;
        Ok(complete_response.completion)
    }
}

/// Get a list of all available Anthropic agents.
///
/// Currently there is not API route to obtain a list of
/// models. Therefore, this uses a static list with versions from
/// https://docs.anthropic.com/claude/reference/selecting-a-model.
///
/// We use full versions of models so that specialized agents
/// can be pinned to a specific version and different specialized
/// agents can use different versions if appropriate.
///
/// Errors if the `ANTHROPIC_API_KEY` env var is not set.
pub async fn list() -> Result<Vec<Arc<dyn Agent>>> {
    if env::var("ANTHROPIC_API_KEY").is_err() {
        bail!("The ANTHROPIC_API_KEY environment variable is not set")
    }

    let models = vec!["claude-instant-1.2", "claude-2.1"];
    let agents: Vec<Arc<dyn Agent>> = models
        .iter()
        .map(|&name| Arc::new(AnthropicAgent::new(name.to_string())) as Arc<dyn Agent>)
        .collect();

    Ok(agents)
}
