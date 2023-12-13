//! Custom Stencila agents specialized for specific tasks
//!
//! An agent is a combination of (a) a model, (b) a default prompt,
//! and (c) a set of default options. This crate defines some specialized
//! agents build on top of lower level, more generalized agents
//! in other crates and prompts defined in the top level `prompts` module.

use std::sync::Arc;

use agent::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    Agent, AgentIO, GenerateOptions,
};
use agent_ollama::OllamaAgent;
use agent_openai::OpenAIAgent;

struct CustomAgent {
    /// The name of the agent
    name: String,

    /// Provider name
    provider: String,

    /// The name of the base provider and model
    model: String,

    /// The name of the default prompt for the agent
    prompt: String,

    /// The set of default options to use for the agent
    #[allow(unused)]
    options: GenerateOptions,

    /// The base agent delegated to
    base: Arc<dyn Agent>,
}

impl CustomAgent {
    fn new(
        name: &str,
        provider: &str,
        model: &str,
        prompt: &str,
        options: GenerateOptions,
    ) -> Result<Self> {
        let base = match provider {
            "openai" => Arc::new(OpenAIAgent::new(
                model.into(),
                vec![AgentIO::Text],
                vec![AgentIO::Text],
            )) as Arc<dyn Agent>,
            "ollama" => Arc::new(OllamaAgent::new(model.into())) as Arc<dyn Agent>,
            _ => bail!("Unknown provider agent: {provider}"),
        };

        Ok(Self {
            name: name.into(),
            provider: provider.into(),
            model: model.into(),
            prompt: prompt.into(),
            options,
            base,
        })
    }
}

#[async_trait]
impl Agent for CustomAgent {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn provider(&self) -> String {
        self.provider.clone()
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn prompt(&self) -> String {
        self.prompt.clone()
    }

    fn supported_inputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    async fn text_to_text(&self, instruction: &str, options: &GenerateOptions) -> Result<String> {
        // TODO: Work out how best to merge supplied options with self.options
        let mut options = options.clone();
        if options.prompt_name.is_none() {
            options.prompt_name = Some(self.prompt.clone());
        }

        self.base.text_to_text(instruction, &options).await
    }
}

/// Get a list of all available custom agents
///
/// Fetches the list of custom models from the server and maps them
/// into agents.
pub async fn list() -> Result<Vec<Arc<dyn Agent>>> {
    let agents = vec![
        Arc::new(CustomAgent::new(
            "custom/insert-block",
            "openai",
            "gpt-3.5-turbo-1106",
            "insert-block",
            GenerateOptions::default(),
        )?) as Arc<dyn Agent>,
        Arc::new(CustomAgent::new(
            "custom/modify-block",
            "openai",
            "gpt-3.5-turbo-1106",
            "modify-block",
            GenerateOptions::default(),
        )?) as Arc<dyn Agent>,
    ];

    Ok(agents)
}
