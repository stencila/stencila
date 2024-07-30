use std::sync::Arc;

use cached::proc_macro::cached;

use model::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
        reqwest::Client,
        serde::{Deserialize, Serialize},
        serde_with::skip_serializing_none,
        tracing,
    },
    schema::{MessagePart, MessageRole},
    secrets, Model, ModelIO, ModelOutput, ModelTask, ModelType,
};

const BASE_URL: &str = "https://api.mistral.ai/v1";

/// The name of the env var or secret for the API key
const API_KEY: &str = "MISTRAL_API_KEY";

struct MistralModel {
    /// The name of the model
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The HTTP client
    client: Client,
}

impl MistralModel {
    /// Create a Mistral model
    fn new(model: &str, context_length: usize) -> Self {
        Self {
            model: model.into(),
            context_length,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Model for MistralModel {
    fn name(&self) -> String {
        format!("mistral/{}", self.model)
    }

    fn r#type(&self) -> ModelType {
        ModelType::Remote
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    #[tracing::instrument(skip(self))]
    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        let messages = task
            .messages
            .iter()
            .map(|message| {
                let role = match message.role.clone().unwrap_or_default() {
                    MessageRole::Assistant => ChatRole::Assistant,
                    MessageRole::System => ChatRole::System,
                    MessageRole::User => ChatRole::User,
                };

                let content = message
                    .parts
                    .iter()
                    .filter_map(|part: &MessagePart| match part {
                        MessagePart::Text(text) => Some(text.to_value_string()),
                        _ => {
                            tracing::warn!(
                                "Message part of type `{part}` is ignored by assistant `{}`",
                                self.name()
                            );
                            None
                        }
                    })
                    .join("");

                ChatMessage { role, content }
            })
            .collect();

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: task.temperature,
            top_p: task.top_p,
            max_tokens: task.max_tokens,
            random_seed: task.seed,
        };

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        let response = self
            .client
            .post(format!("{BASE_URL}/chat/completions"))
            .bearer_auth(secrets::env_or_get(API_KEY)?)
            .json(&request)
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            bail!(error);
        }

        let mut response: ChatCompletionResponse = response.json().await?;

        let text = response.choices.swap_remove(0).message.content;

        ModelOutput::from_text(self, &task.format, text).await
    }
}

/// A model list response
///
/// Based on https://docs.mistral.ai/api#operation/listModels
#[derive(Deserialize)]
#[serde(crate = "model::common::serde")]
struct ModelsResponse {
    data: Vec<ModelSpec>,
}

/// A model returned within a `ModelsResponse`
///
/// Note: at present several other fields are ignored.
#[derive(Deserialize)]
#[serde(crate = "model::common::serde")]
struct ModelSpec {
    id: String,
}

/// A chat completion request
///
/// Based on https://docs.mistral.ai/api#operation/createChatCompletion
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(crate = "model::common::serde")]
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
#[serde(crate = "model::common::serde")]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

/// A choice within a `ChatCompletionResponse`
///
/// Note: at present several other fields are ignored.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(crate = "model::common::serde")]
struct ChatCompletionChoice {
    message: ChatMessage,
}

/// A chat message within a `ChatCompletionRequest` or a `ChatCompletionResponse`
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(crate = "model::common::serde")]
struct ChatMessage {
    role: ChatRole,
    content: String,
}

/// A role in a `ChatMessage`
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "model::common::serde")]
enum ChatRole {
    System,
    User,
    Assistant,
}

/// Get a list of available Mistral assistants
///
/// Returns an empty list if the `MISTRAL_API_KEY` env var is not set.
///
/// Memoized for an hour to reduce the number of times that the
/// remote API need to be called to get a list of available models.
#[cached(time = 3600, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    let Ok(key) = secrets::env_or_get(API_KEY) else {
        tracing::debug!("The environment variable or secret `{API_KEY}` is not available");
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

    let models = models
        .into_iter()
        .filter(|ModelSpec { id: model }| {
            // Only include models with numeric version (not un-versioned or latest)
            let parts = model.split('-').collect_vec();
            ((model.starts_with("codestral") && parts.len() == 2) || parts.len() >= 3)
                && parts
                    .last()
                    .map(|&version| version.starts_with('2'))
                    .unwrap_or(false)
        })
        .sorted_by(|a, b| a.id.cmp(&b.id))
        .map(|ModelSpec { id: model }| {
            let (name, _version) = model.rsplit_once('-').unwrap_or_default();
            let context_length = match name {
                "mistral-tiny" => 4_096,
                "mistral-small" => 8_192,
                "mistral-medium" => 32_768,
                "mistral-large" => 128_000,
                _ => 4_096,
            };

            Arc::new(MistralModel::new(&model, context_length)) as Arc<dyn Model>
        })
        .collect();

    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{common::tokio, test_task_repeat_word};

    #[tokio::test]
    async fn list_models() -> Result<()> {
        let list = list().await?;

        if secrets::env_or_get(API_KEY).is_err() {
            assert_eq!(list.len(), 0)
        } else {
            assert!(!list.is_empty())
        }

        Ok(())
    }

    #[tokio::test]
    async fn perform_task() -> Result<()> {
        if secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = MistralModel::new("mistral-large-latest", 0);
        let output = model.perform_task(&test_task_repeat_word()).await?;

        assert_eq!(output.content.trim(), "HELLO");

        Ok(())
    }
}
