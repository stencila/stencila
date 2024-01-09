use std::{env, sync::Arc};

use anthropic::{
    client::Client, config::AnthropicConfig, types::CompleteRequestBuilder, AI_PROMPT, HUMAN_PROMPT,
};

use assistant::{
    common::{async_trait::async_trait, eyre::Result, itertools::Itertools, tracing},
    schema::{MessagePart, PersonOrOrganizationOrSoftwareApplication},
    Assistant, AssistantIO, GenerateOptions, GenerateOutput, GenerateTask,
};

const API_KEY: &str = "ANTHROPIC_API_KEY";

/// An assistant running on Anthropic
///
/// The environment variable ANTHROPIC_API_KEY must be set to use these assistants.
pub struct AnthropicAssistant {
    /// The name of the model including its version e.g. "claude-2.1"
    model: String,

    /// The context length of the model
    context_length: usize,
}

impl AnthropicAssistant {
    /// Create an Anthropic assistant
    fn new(model: String, context_length: usize) -> Self {
        Self {
            model,
            context_length,
        }
    }
}

#[async_trait]
impl Assistant for AnthropicAssistant {
    fn id(&self) -> String {
        format!("anthropic/{}", self.model)
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        // TODO: This does not use the new Messages API and instead concatenates messages into a chat string
        // https://docs.anthropic.com/claude/reference/messages_post

        let system_prompt = match &task.system_prompt {
            Some(prompt) => prompt.clone(),
            None => String::new(),
        };

        let chat = task
            .instruction_messages()
            .map(|message| {
                use PersonOrOrganizationOrSoftwareApplication::*;
                let prompt = match message.sender {
                    None | Some(Person(..) | Organization(..)) => HUMAN_PROMPT,
                    Some(SoftwareApplication(..)) => AI_PROMPT,
                };

                let content = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::String(text) => Some(text),
                        _ => {
                            tracing::warn!(
                                "User message part `{part}` is ignored by assistant `{}`",
                                self.id()
                            );
                            None
                        }
                    })
                    .join("");

                format!("{prompt}{content}")
            })
            .join("\n\n");

        // With the Completions API system prompts are just put before the chat
        // https://docs.anthropic.com/claude/docs/how-to-use-system-prompts
        let prompt = format!("{system_prompt}{chat}{AI_PROMPT}");

        // Build completion request from `options`
        // TODO: We need to add the following which are not in the CompleteRequest (maybe by PR).
        // https://docs.anthropic.com/claude/reference/complete_post
        // temperature
        // top_k
        // top_p
        let complete_request = CompleteRequestBuilder::default()
            .model(&self.model)
            .prompt(prompt)
            // Not sure the best way to do this, but 256 is the default.
            .max_tokens_to_sample(options.max_tokens.unwrap_or(256) as usize)
            .stop_sequences(vec![HUMAN_PROMPT.to_string()])
            .build()?;

        let cfg = AnthropicConfig::new()?;
        let client = Client::try_from(cfg)?;

        let text = client
            .complete(complete_request)
            .await?
            .completion
            .trim_start()
            .to_string();

        GenerateOutput::from_text(self, task, options, text).await
    }
}

/// Get a list of all available Anthropic assistants.
///
/// Currently there is no Anthropic API route to obtain a list of models.
/// Therefore, this uses a static list with versions and other info from
/// https://docs.anthropic.com/claude/reference/input-and-output-sizes.
///
/// If the `ANTHROPIC_API_KEY` env var is not set returns an empty list.
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    if env::var(API_KEY).is_err() {
        tracing::debug!("The {API_KEY} environment variable is not set");
        return Ok(vec![]);
    }

    let assistants = [
        ("claude-2.1", 200_000),
        ("claude-2.0", 100_000),
        ("claude-instant-1.2", 100_000),
    ]
    .into_iter()
    .map(|(model, context_length)| {
        Arc::new(AnthropicAssistant::new(model.to_string(), context_length)) as Arc<dyn Assistant>
    })
    .collect();

    Ok(assistants)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assistant::{common::tokio, test_task_repeat_word, GenerateContent};

    #[tokio::test]
    async fn list_assistants() -> Result<()> {
        let list = list().await?;

        if env::var(API_KEY).is_err() {
            assert_eq!(list.len(), 0)
        } else {
            assert!(!list.is_empty())
        }

        Ok(())
    }

    #[tokio::test]
    async fn perform_task() -> Result<()> {
        if env::var(API_KEY).is_err() {
            return Ok(());
        }

        let assistant = &list().await?[0];
        let output = assistant
            .perform_task(&test_task_repeat_word(), &GenerateOptions::default())
            .await?;

        assert_eq!(output.content, GenerateContent::Text("HELLO".to_string()));

        Ok(())
    }
}
