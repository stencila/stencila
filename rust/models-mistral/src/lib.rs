use cached::proc_macro::cached;
use inflector::Inflector;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{sync::Arc, time::Duration};
use stencila_node_media::embed_image;

use stencila_model::{
    Model, ModelIO, ModelOutput, ModelTask, ModelType, async_trait,
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::{File, MessagePart, MessageRole},
    stencila_schema_json::JsonSchema,
    stencila_secrets,
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

const MAX_ERROR_MESSAGE_LEN: usize = 1000;

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
        self.model
            .split('-')
            .next_back()
            .unwrap_or_default()
            .to_string()
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

    #[tracing::instrument(skip_all)]
    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        if self.id().contains("-ocr") {
            self.ocr(task).await
        } else {
            self.chat_completion(task).await
        }
    }
}

impl MistralModel {
    /// Make a Chat Completion API request
    ///
    /// See https://docs.mistral.ai/api/#tag/chat
    #[tracing::instrument(skip_all)]
    async fn chat_completion(&self, task: &ModelTask) -> Result<ModelOutput> {
        let mut messages = Vec::new();
        for message in &task.messages {
            let role = match message.role.unwrap_or_default() {
                MessageRole::Model => ChatRole::Assistant,
                MessageRole::System => ChatRole::System,
                MessageRole::User => ChatRole::User,
            };

            let mut items = Vec::new();
            for part in &message.parts {
                let item = match part {
                    MessagePart::Text(text) => ChatMessageContentItem::Text {
                        text: text.value.to_string(),
                    },
                    MessagePart::ImageObject(image) => {
                        let image_url = if image.content_url.starts_with("data:") {
                            image.content_url.to_string()
                        } else {
                            let mut image = image.clone();
                            embed_image(&mut image, None, None)?;
                            image.content_url
                        };
                        ChatMessageContentItem::ImageUrl { image_url }
                    }
                    _ => bail!("Unexpected message part"),
                };
                items.push(item)
            }

            let content = ChatMessageContent::Array(items);

            messages.push(ChatMessage { role, content })
        }

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            response_format: task
                .schema
                .is_some()
                .then(|| ResponseFormat::from_task(task)),
            temperature: task.temperature,
            top_p: task.top_p,
            max_tokens: task.max_tokens,
            random_seed: task.seed,
        };

        if task.dry_run {
            return Ok(ModelOutput::empty(self));
        }

        let response = self
            .client
            .post(format!("{BASE_URL}/chat/completions"))
            .bearer_auth(stencila_secrets::env_or_get(API_KEY)?)
            .json(&request)
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            let mut message = response.text().await?;
            message.truncate(MAX_ERROR_MESSAGE_LEN);
            bail!("{error}: {message}");
        }
        let mut response: ChatCompletionResponse = response.json().await?;

        match response.choices.swap_remove(0).message.content {
            ChatMessageContent::String(content) => {
                Ok(ModelOutput::from_text(self, &task.format, content))
            }
            ChatMessageContent::Array(mut items) => match items.swap_remove(0) {
                ChatMessageContentItem::Text { text } => {
                    Ok(ModelOutput::from_text(self, &task.format, text))
                }
                ChatMessageContentItem::ImageUrl { image_url } => {
                    Ok(ModelOutput::from_url(self, &task.format, image_url))
                }
            },
        }
    }

    /// Make an OCR API request
    ///
    /// See https://docs.mistral.ai/api/#tag/ocr
    #[tracing::instrument(skip_all)]
    async fn ocr(&self, task: &ModelTask) -> Result<ModelOutput> {
        let part = task
            .messages
            .iter()
            .flat_map(|message| &message.parts)
            .find(|part| matches!(part, MessagePart::File(..) | MessagePart::ImageObject(..)))
            .ok_or_else(|| eyre!("No file or image found in message parts"))?;

        let document = match part {
            MessagePart::File(file) => {
                let Some(document_url) = file.to_data_uri() else {
                    bail!("File appears to be empty")
                };
                OcrDocument::DocumentUrl { document_url }
            }
            MessagePart::ImageObject(image) => {
                let image_url = if image.content_url.starts_with("data:") {
                    image.content_url.to_string()
                } else {
                    let mut image = image.clone();
                    embed_image(&mut image, None, None)?;
                    image.content_url
                };
                OcrDocument::ImageUrl { image_url }
            }
            _ => unreachable!(),
        };

        let request = OcrRequest {
            model: self.model.clone(),
            document,
            include_image_base64: Some(true),
            document_annotation_format: task
                .schema
                .is_some()
                .then(|| ResponseFormat::from_task(task)),
        };

        if task.dry_run {
            return Ok(ModelOutput::empty(self));
        }

        let response = self
            .client
            .post(format!("{BASE_URL}/ocr"))
            .bearer_auth(stencila_secrets::env_or_get(API_KEY)?)
            .json(&request)
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            let mut message = response.text().await?;
            message.truncate(MAX_ERROR_MESSAGE_LEN);
            bail!("{error}: {message}");
        }
        let response: OcrResponse = response.json().await?;

        let mut markdown = if let Some(json) = response.document_annotation {
            // If any metadata were extract then add as YAML frontmatter
            let metadata: serde_json::Value = serde_json::from_str(&json)?;
            let yaml = serde_yaml::to_string(&metadata)?;
            format!("---\n{yaml}\n---\n")
        } else {
            String::new()
        };

        let mut attachments = Vec::new();

        for page in response.pages {
            // Note: only one newline here to avoid splitting paragraphs unnecessarily
            markdown.push('\n');
            markdown.push_str(&page.markdown);

            for image in page.images {
                if let Some(image_base64) = image.image_base64 {
                    // Extract content after "data:image/...;base64," prefix
                    if let Some(content_start) = image_base64.find(";base64,") {
                        let content = image_base64[content_start + 8..].to_string();

                        let media_type = if image_base64.starts_with("data:image/") {
                            image_base64[5..].split(';').next().map(String::from)
                        } else {
                            None
                        };

                        attachments.push(File {
                            name: image.id.clone(),
                            media_type,
                            content: Some(content),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        Ok(ModelOutput::from_text_with(
            self,
            &Some(Format::Markdown),
            markdown,
            attachments,
        ))
    }
}

/// A model list response
///
/// Based on https://docs.mistral.ai/api#operation/listModels
#[derive(Clone, Serialize, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelSpec>,
}

/// A model returned within a `ModelsResponse`
///
/// Note: at present several other fields are ignored.
#[derive(Clone, Serialize, Deserialize)]
struct ModelSpec {
    id: String,
}

/// A chat completion request
///
/// Based on https://docs.mistral.ai/api#operation/createChatCompletion
#[skip_serializing_none]
#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    response_format: Option<ResponseFormat>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_tokens: Option<u16>,
    random_seed: Option<i32>,
}

/// The requested response format
#[derive(Serialize)]
struct ResponseFormat {
    r#type: ResponseFormatType,
    json_schema: Option<ResponseJsonSchema>,
}

/// The requested response format type
#[derive(Default, Serialize)]
#[serde(rename_all = "snake_case")]
enum ResponseFormatType {
    #[default]
    Text,
    JsonObject,
    JsonSchema,
}

/// The requested response schema
#[derive(Serialize)]
struct ResponseJsonSchema {
    name: String,
    schema: JsonSchema,
    strict: bool,
}

impl ResponseFormat {
    fn from_task(task: &ModelTask) -> Self {
        Self {
            r#type: match task.format {
                Some(Format::Json) => {
                    if task.schema.is_some() {
                        ResponseFormatType::JsonSchema
                    } else {
                        ResponseFormatType::JsonObject
                    }
                }
                _ => ResponseFormatType::Text,
            },
            json_schema: task.schema.clone().map(|schema| ResponseJsonSchema {
                name: "response_schema".to_string(),
                schema: schema.for_mistral(),
                strict: true,
            }),
        }
    }
}

/// A chat completion response
///
/// Note: at present several other fields are ignored.
#[skip_serializing_none]
#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

/// A choice within a `ChatCompletionResponse`
///
/// Note: at present several other fields are ignored.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessage,
}

/// A chat message within a `ChatCompletionRequest` or a `ChatCompletionResponse`
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: ChatRole,
    content: ChatMessageContent,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ChatMessageContent {
    String(String),
    Array(Vec<ChatMessageContentItem>),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ChatMessageContentItem {
    Text { text: String },
    ImageUrl { image_url: String },
}

/// A role in a `ChatMessage`
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ChatRole {
    System,
    User,
    Assistant,
}

/// An OCR request
#[skip_serializing_none]
#[derive(Serialize)]
struct OcrRequest {
    model: String,
    document: OcrDocument,
    include_image_base64: Option<bool>,
    document_annotation_format: Option<ResponseFormat>,
}

/// An OCR document
#[derive(Serialize)]
#[serde(untagged)]
enum OcrDocument {
    DocumentUrl { document_url: String },
    ImageUrl { image_url: String },
}

/// An OCR response
#[derive(Deserialize)]
struct OcrResponse {
    pages: Vec<OcrPage>,
    document_annotation: Option<String>,
}

/// An OCR page response
#[derive(Deserialize)]
struct OcrPage {
    #[allow(dead_code)]
    index: usize,
    markdown: String,
    #[serde(default)]
    images: Vec<OcrImage>,
}

/// An OCR image
#[allow(dead_code)]
#[derive(Deserialize)]
struct OcrImage {
    id: String,
    top_left_x: f64,
    top_left_y: f64,
    bottom_right_x: f64,
    bottom_right_y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_base64: Option<String>,
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
    if stencila_secrets::env_or_get(API_KEY).is_err() {
        tracing::trace!("The environment variable or secret `{API_KEY}` is not available");
        return Ok(vec![]);
    };

    let models = list_mistral_models()
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
/// In-memory cached for six hours to reduce requests to remote API.
#[cached(time = 21_600, result = true)]
async fn list_mistral_models() -> Result<ModelsResponse> {
    let response = Client::new()
        .get(format!("{BASE_URL}/models"))
        .bearer_auth(stencila_secrets::env_or_get(API_KEY)?)
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
    use stencila_model::test_task_repeat_word;

    #[tokio::test]
    async fn list_models() -> Result<()> {
        let list = list().await?;

        if stencila_secrets::env_or_get(API_KEY).is_err() {
            assert_eq!(list.len(), 0)
        } else {
            assert!(!list.is_empty())
        }

        Ok(())
    }

    #[tokio::test]
    async fn perform_task() -> Result<()> {
        if stencila_secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = MistralModel::new("mistral-large-latest", 0);
        let output = model.perform_task(&test_task_repeat_word()).await?;

        assert_eq!(output.content.trim(), "HELLO");

        Ok(())
    }
}
