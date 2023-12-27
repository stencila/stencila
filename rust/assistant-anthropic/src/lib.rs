use std::{env, sync::Arc};

use anthropic::{
    client::Client, config::AnthropicConfig, types::CompleteRequestBuilder, AI_PROMPT, HUMAN_PROMPT,
};

use assistant::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    Assistant, AssistantIO, GenerateDetails, GenerateOptions, GenerateOutput, GenerateTask,
};

/// An assistant running on Anthropic
///
/// The environment variable ANTHROPIC_API_KEY must be set to use these assistants.
pub struct AnthropicAssistant {
    /// The name of the model including its version e.g. "claude-2.1"
    model: String,
}

impl AnthropicAssistant {
    /// Create a Anthropic assistant
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

#[async_trait]
impl Assistant for AnthropicAssistant {
    fn provider(&self) -> String {
        "anthropic".to_string()
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    async fn perform_task(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(GenerateOutput, GenerateDetails)> {
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
                "{system_prompt} {HUMAN_PROMPT}{user_prompt}{AI_PROMPT}",
                system_prompt = task.system_prompt().unwrap_or(""),
                user_prompt = task.user_prompt()
            ))
            .model(&self.model)
            // Not sure the best way to do this, but 256 is the default.
            .max_tokens_to_sample(options.max_tokens.unwrap_or(256) as usize)
            .stop_sequences(vec![HUMAN_PROMPT.to_string()])
            .build()?;

        let text = client.complete(complete_request).await?.completion;
        let output = GenerateOutput::Text(text);

        let details = GenerateDetails {
            task,
            options: options.clone(),
            assistants: vec![self.name()],
            ..Default::default()
        };

        Ok((output, details))
    }
}

/// Get a list of all available Anthropic assistants.
///
/// Currently there is not API route to obtain a list of
/// models. Therefore, this uses a static list with versions from
/// https://docs.anthropic.com/claude/reference/selecting-a-model.
///
/// We use full versions of models so that specialized assistants
/// can be pinned to a specific version and different specialized
/// assistants can use different versions if appropriate.
///
/// Errors if the `ANTHROPIC_API_KEY` env var is not set.
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    if env::var("ANTHROPIC_API_KEY").is_err() {
        bail!("The ANTHROPIC_API_KEY environment variable is not set")
    }

    let models = vec!["claude-instant-1.2", "claude-2.1"];
    let assistants: Vec<Arc<dyn Assistant>> = models
        .iter()
        .map(|&name| Arc::new(AnthropicAssistant::new(name.to_string())) as Arc<dyn Assistant>)
        .collect();

    Ok(assistants)
}
