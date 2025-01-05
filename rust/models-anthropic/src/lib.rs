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
    schema::{ImageObject, MessagePart, MessageRole},
    secrets, Model, ModelIO, ModelOutput, ModelTask, ModelType,
};

/// The base URL for the Anthropic API
const BASE_URL: &str = "https://api.anthropic.com/v1";

/// The version of the Anthropic API used
const API_VERSION: &str = "2023-06-01";

/// The name of the env var or secret for the API key
const API_KEY: &str = "ANTHROPIC_API_KEY";

/// An model running on Anthropic
///
/// See https://docs.anthropic.com/en/api/messages for the API that this
/// is targeting.
pub struct AnthropicModel {
    /// The name of the model including its version
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The HTTP client for accessing the Anthropic API
    client: Client,
}

impl AnthropicModel {
    /// Create an Anthropic model
    fn new(model: &str, context_length: usize) -> Self {
        Self {
            model: model.into(),
            context_length,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Model for AnthropicModel {
    fn id(&self) -> String {
        format!("anthropic/{}", self.model)
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

    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        let mut system = None;
        let messages = task
            .messages
            .iter()
            .filter_map(|message| {
                if matches!(message.role, Some(MessageRole::System)) {
                    let text = message
                        .parts
                        .iter()
                        .filter_map(|part| match part {
                            MessagePart::Text(text) => Some(text.to_value_string()),
                            _ => {
                                tracing::warn!(
                                    "System message part `{part}` is ignored by model `{}`",
                                    self.id()
                                );
                                None
                            }
                        })
                        .join("\n");
                    system = Some(text);

                    return None;
                }

                let role = match message.role {
                    Some(MessageRole::Model) => "assistant",
                    Some(MessageRole::System) => "system",
                    _ => "user",
                }
                .to_string();

                let content = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::Text(text) => Some(ContentPart::Text {
                            text: text.to_value_string(),
                        }),
                        MessagePart::ImageObject(ImageObject{content_url,..}) =>
                            if let (true, Some(pos)) = (content_url.starts_with("data:"), content_url.find(";base64,")) {
                                let media_type = content_url[5..pos].to_string();
                                let data = content_url[(pos + 8)..].to_string();

                                Some(ContentPart::Image {
                                    source: ImageSource {
                                        r#type: "base64".into(),
                                        media_type,
                                        data,
                                    },
                                })
                            } else {
                                tracing::warn!(
                                    "Image does not appear to have a DataURI so was ignored by model `{}`",
                                    self.id()
                                );
                                None
                            }
                        _ => {
                            tracing::warn!(
                                "User message part `{part}` is ignored by model `{}`",
                                self.id()
                            );
                            None
                        }
                    })
                    .collect_vec();

                Some(Message { role, content })
            })
            .collect_vec();

        let request = MessagesRequest {
            model: self.model.clone(),
            messages,
            system,
            // Required parameter. See here for a list of max supported by each model
            // https://docs.anthropic.com/en/docs/about-claude/models#model-comparison
            max_tokens: task.max_tokens.unwrap_or(4096),
            temperature: task.temperature,
            top_k: task.top_k,
            top_p: task.top_p,
        };

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        let response = self
            .client
            .post(format!("{BASE_URL}/messages/"))
            .header("x-api-key", secrets::env_or_get(API_KEY)?)
            .header("anthropic-version", API_VERSION)
            .json(&request)
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            let message = response.text().await?;
            bail!("{error}: {message}");
        }

        let response: MessagesResponse = response.json().await?;

        let text = response
            .content
            .into_iter()
            .filter_map(|part| match part {
                ContentPart::Text { text } => Some(text),
                _ => None,
            })
            .join("\n\n");

        ModelOutput::from_text(self, &task.format, text).await
    }
}

/// Get a list of all available Anthropic models.
///
/// Currently there is no Anthropic API route to obtain a list of models.
/// Therefore, this uses a static list with versions and other info from
/// https://docs.anthropic.com/claude/reference/input-and-output-sizes.
///
/// If the Anthropic API key is not available returns an empty list.
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    if secrets::env_or_get(API_KEY).is_err() {
        tracing::trace!("The environment variable or secret `{API_KEY}` is not available");
        return Ok(vec![]);
    }

    let models = list_anthropic_models()
        .await?
        .data
        .into_iter()
        .sorted_by(|a, b| a.id.cmp(&b.id))
        .map(|ModelSpec { id: model }| {
            Arc::new(AnthropicModel::new(&model, 200_000)) as Arc<dyn Model>
        })
        .collect();

    Ok(models)
}

/// Fetch the list of models
///
/// In-memory cached for six hours to reduce requests to remote API.
#[cached(time = 21_600, result = true)]
async fn list_anthropic_models() -> Result<ModelsResponse> {
    let response = Client::new()
        .get(format!("{}/models", BASE_URL))
        .header("x-api-key", secrets::env_or_get(API_KEY)?)
        .header("anthropic-version", API_VERSION)
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        let message = response.text().await?;
        bail!("{error}: {message}");
    }

    Ok(response.json().await?)
}

/// A model list response
///
/// Based on https://docs.anthropic.com/en/api/models-list
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

/// A part within the content of a message in the Messages API
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", crate = "model::common::serde")]
enum ContentPart {
    Text { text: String },
    Image { source: ImageSource },
}

/// An images source
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "model::common::serde")]
struct ImageSource {
    r#type: String,
    media_type: String,
    data: String,
}

/// A Messages API message
///
/// Note: at present only text content is handled
#[derive(Debug, Serialize)]
#[serde(crate = "model::common::serde")]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

/// A Messages API request body
///
/// Based on https://docs.anthropic.com/en/api/messages.
/// Note: at present several fields are ignored.
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(crate = "model::common::serde")]
struct MessagesRequest {
    model: String,
    messages: Vec<Message>,
    system: Option<String>,
    max_tokens: u16,
    temperature: Option<f32>,
    top_k: Option<u32>,
    top_p: Option<f32>,
}

/// A Messages API response body
///
/// Based on https://docs.anthropic.com/en/api/messages.
/// Note: at present several fields are ignored.
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(crate = "model::common::serde")]
struct MessagesResponse {
    content: Vec<ContentPart>,
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

        let model = AnthropicModel::new("claude-3-5-sonnet-20240620", 0);
        let output = model.perform_task(&test_task_repeat_word()).await?;

        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }
}
