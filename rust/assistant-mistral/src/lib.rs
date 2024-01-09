use std::{env, sync::Arc};

use cached::proc_macro::cached;

use assistant::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
        reqwest::Client,
        serde::{Deserialize, Serialize},
        serde_with::skip_serializing_none,
        tracing,
    },
    schema::{MessagePart, PersonOrOrganizationOrSoftwareApplication},
    Assistant, AssistantIO, GenerateOptions, GenerateOutput, GenerateTask,
};

const BASE_URL: &str = "https://api.mistral.ai/v1";
const API_KEY: &str = "MISTRAL_API_KEY";

struct MistralAssistant {
    /// The name of the model
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The HTTP client
    client: Client,
}

impl MistralAssistant {
    /// Create a Mistral assistant
    fn new(model: String, context_length: usize) -> Self {
        Self {
            model,
            context_length,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Assistant for MistralAssistant {
    fn id(&self) -> String {
        format!("mistral/{}", self.model)
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

    #[tracing::instrument(skip(self))]
    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        let messages = task
            .system_prompt
            .iter()
            .map(|prompt| ChatMessage {
                role: ChatRole::System,
                content: prompt.clone(),
            })
            .chain(task.instruction_messages().map(|message| {
                use PersonOrOrganizationOrSoftwareApplication::*;
                let role = match message.sender {
                    None | Some(Person(..) | Organization(..)) => ChatRole::User,
                    Some(SoftwareApplication(..)) => ChatRole::Assistant,
                };

                let content = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::String(text) => Some(text),
                        _ => {
                            tracing::warn!(
                                "Message part of type `{part}` is ignored by assistant `{}`",
                                self.id()
                            );
                            None
                        }
                    })
                    .join("");

                ChatMessage { role, content }
            }))
            .collect();

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: options.temperature,
            top_p: options.top_p,
            max_tokens: options.max_tokens,
            random_seed: options.seed,
        };

        let response = self
            .client
            .post(format!("{BASE_URL}/chat/completions"))
            .bearer_auth(env::var(API_KEY)?)
            .json(&request)
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            bail!(error);
        }

        let mut response: ChatCompletionResponse = response.json().await?;

        let text = response.choices.swap_remove(0).message.content;

        GenerateOutput::from_text(self, task, options, text).await
    }
}

/// A model list response
///
/// Based on https://docs.mistral.ai/api#operation/listModels
#[derive(Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct ModelsResponse {
    data: Vec<Model>,
}

/// A model returned within a `ModelsResponse`
///
/// Note: at present several other fields are ignored.
#[derive(Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct Model {
    id: String,
}

/// A chat completion request
///
/// Based on https://docs.mistral.ai/api#operation/createChatCompletion
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(crate = "assistant::common::serde")]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_tokens: Option<u16>,
    random_seed: Option<i32>,
}

/// A chat completion response
///
/// Note: at present several other fields are ignored.
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

/// A choice within a `ChatCompletionResponse`
///
/// Note: at present several other fields are ignored.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct ChatCompletionChoice {
    message: ChatMessage,
}

/// A chat message within a `ChatCompletionRequest` or a `ChatCompletionResponse`
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct ChatMessage {
    role: ChatRole,
    content: String,
}

/// A role in a `ChatMessage`
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "assistant::common::serde")]
enum ChatRole {
    System,
    User,
    Assistant,
}

/// Get a list of available Mistral assistants
///
/// Returns an empty list, if the `MISTRAL_API_KEY` env var is not set.
///
/// Memoized for an hour to reduce the number of times that the
/// remote API need to be called to get a list of available models.
#[cached(time = 3600, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    let Ok(key) = env::var(API_KEY) else {
        tracing::debug!("The {API_KEY} environment variable is not set");
        return Ok(vec![]);
    };

    let response = Client::new()
        .get(format!("{}/models", BASE_URL))
        .bearer_auth(key)
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!(error);
    }

    let ModelsResponse { data: models } = response.json().await?;

    let assistants = models
        .into_iter()
        .filter(|model| !model.id.ends_with("-embed"))
        .sorted_by(|a, b| a.id.cmp(&b.id))
        .map(|Model { id: model }| {
            let context_length = match model.as_str() {
                "mistral-tiny" => 4_096,
                "mistral-small" => 8_192,
                "mistral-medium" => 32_768,
                _ => 4_096,
            };

            Arc::new(MistralAssistant::new(model, context_length)) as Arc<dyn Assistant>
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

        let text = match output.content {
            GenerateContent::Text(text) => text,
            _ => bail!("Expected text content"),
        };
        assert!(text.starts_with("HELLO"));

        Ok(())
    }
}
