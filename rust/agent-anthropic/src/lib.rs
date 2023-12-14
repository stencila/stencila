use std::{env, sync::Arc};

use anthropic::{
    client::Client,
    config::AnthropicConfig,
    types::CompleteRequestBuilder,
    AI_PROMPT, HUMAN_PROMPT,
};


use agent::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    Agent, AgentIO, GenerateOptions,
};

/// An agent running on OpenAI
///
/// The environment variable OPENAI_API_KEY must be set to use these agents.
pub struct AnthropicAgent {
    /// The OpenAI name for a model including any tag e.g. "llama2:13b"
    ///
    /// Used as the required `model` parameter in each request to `POST /api/generate`
    /// (along with `prompt`).
    model: String,

}

impl AnthropicAgent {
    /// Create a OpenAI-based agent
    pub fn new(model: String) -> Self {
        Self {
            model,
        }
    }
   fn supported_inputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
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

    // fn supported_inputs(&self) -> &[AgentIO] {
    //     &[AgentIO::Text]
    // }

    // fn supported_outputs(&self) -> &[AgentIO] {
    //     &[AgentIO::Text]
    // }

    async fn text_to_text(&self, instruction: &str, options: &GenerateOptions) -> Result<String> {
        // Build from configuration.
        // TODO: We need to use the configuration here.
        // Currently just a cut/paste from the example.
        let cfg = AnthropicConfig::new()?;
        let client = Client::try_from(cfg)?;

        let complete_request = CompleteRequestBuilder::default()
            .prompt(format!("{HUMAN_PROMPT}{instruction}{AI_PROMPT}"))
            // What is claude-instant-1??
            .model("claude-instant-1".to_string())
            .max_tokens_to_sample(256usize)
            .stream_response(false)
            .stop_sequences(vec![HUMAN_PROMPT.to_string()])
            .build()?;

        // Send a completion request.
        let complete_response = client.complete(complete_request).await?;
        Ok(complete_response.completion)
    }
}

/// Get a list of all available Anthropic agents. There is ONLY ONE!
///
/// Errors if the `ANTHROPIC_API_KEY` env var is not set.
pub async fn list() -> Result<Vec<Arc<dyn Agent>>> {
    if env::var("ANTHROPIC_API_KEY").is_err() {
        bail!("The ANTHROPIC_API_KEY environment variable is not set")
    }
    let agent = AnthropicAgent::new("claude-v1".to_string());
    let agents = vec![Arc::new(agent) as Arc<dyn Agent>];
    Ok(agents)
}
