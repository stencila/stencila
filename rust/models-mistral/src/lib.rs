use std::sync::Arc;

use cached::proc_macro::{cached, io_cached};

use model::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        inflector::Inflector,
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
    fn id(&self) -> String {
        format!("mistral/{}", self.model)
    }

    fn r#type(&self) -> ModelType {
        ModelType::Remote
    }

    fn name(&self) -> String {
        if self.model.starts_with("open-mistral-nemo") {
            "Mistral Nemo".to_string()
        } else if self.model.starts_with("open-mixtral") {
            "Mixtral".to_string()
        } else {
            let parts = self.model.split('-').collect_vec();
            if parts.len() > 2 {
                parts.iter().take(2).join(" ").to_title_case()
            } else {
                parts[0].to_title_case()
            }
        }
    }

    fn version(&self) -> String {
        self.model.split('-').last().unwrap_or_default().to_string()
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
                                "Message part of type `{part}` is ignored by model `{}`",
                                self.id()
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
            let message = response.text().await?;
            bail!("{error}: {message}");
        }

        let mut response: ChatCompletionResponse = response.json().await?;

        let text = response.choices.swap_remove(0).message.content;

        ModelOutput::from_text(self, &task.format, text).await
    }
}

/// A model list response
///
/// Based on https://docs.mistral.ai/api#operation/listModels
#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "model::common::serde")]
struct ModelsResponse {
    data: Vec<ModelSpec>,
}

/// A model returned within a `ModelsResponse`
///
/// Note: at present several other fields are ignored.
#[derive(Clone, Serialize, Deserialize)]
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

/// Get a list of available Mistral models
///
/// Returns an empty list if the `MISTRAL_API_KEY` env var is not set.
///
/// Memoized for two minutes to avoid loading from disk cache too frequently
/// but allowing user to set API key while process is running.
#[cached(time = 120, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    // Check for API key before calling IO cached function so that we never cache an empty list
    // and allow for users to set key, and then get list, while process is running
    if secrets::env_or_get(API_KEY).is_err() {
        tracing::trace!("The environment variable or secret `{API_KEY}` is not available");
        return Ok(vec![]);
    };

    let models = list_mistral_models(0)
        .await?
        .data
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

/// Fetch the list of models
///
/// Disk cached for six hours to reduce calls to remote API.
/// For some reason the `io_cached` macro requires at least one function argument.
#[io_cached(disk = true, time = 21_600, map_error = r##"|e| eyre!(e)"##)]
async fn list_mistral_models(_unused: u8) -> Result<ModelsResponse> {
    let response = Client::new()
        .get(format!("{}/models", BASE_URL))
        .bearer_auth(secrets::env_or_get(API_KEY)?)
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        let message = response.text().await?;
        bail!("{error}: {message}");
    }

    Ok(response.json().await?)
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
