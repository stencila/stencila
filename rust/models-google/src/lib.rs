use std::{sync::Arc, time::Duration};

use cached::proc_macro::cached;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use stencila_model::{
    Model, ModelIO, ModelOutput, ModelTask, ModelType, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::{MessagePart, MessageRole},
    stencila_schema_json::JsonSchema,
    stencila_secrets,
};
use stencila_node_media::embed_image;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// The name of the env var or secret for the API key
const API_KEY: &str = "GOOGLE_AI_API_KEY";

struct GoogleModel {
    /// The name of the model
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The HTTP client for accessing the Google AI API
    client: Client,
}

impl GoogleModel {
    /// Create a Google AI model
    fn new(model: &str, context_length: usize) -> Self {
        Self {
            model: model.into(),
            context_length,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Model for GoogleModel {
    fn id(&self) -> String {
        format!("google/{}", self.model)
    }

    fn r#type(&self) -> ModelType {
        ModelType::Remote
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        use ModelIO::*;
        match self.model.as_str() {
            "gemini-pro-vision" => &[Text, Video],
            _ => &[Text],
        }
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    #[tracing::instrument(skip_all)]
    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        let mut system_instruction = None;
        let mut contents = Vec::new();
        for message in task.messages.iter() {
            if matches!(message.role, Some(MessageRole::System)) {
                let parts = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::Text(text) => Some(Part::text(&text.value)),
                        _ => {
                            tracing::warn!(
                                "System message part `{part}` is ignored by model `{}`",
                                self.id()
                            );
                            None
                        }
                    })
                    .collect_vec();
                system_instruction = Some(Content { role: None, parts });
                continue;
            }

            let role = match message.role.unwrap_or_default() {
                MessageRole::Model => Some(Role::Model),
                MessageRole::User => Some(Role::User),
                _ => None,
            };

            let mut parts = Vec::new();
            for part in message.parts.iter() {
                match part {
                    MessagePart::Text(text) => parts.push(Part::text(&text.value)),
                    MessagePart::ImageObject(image) => {
                        let content_url = if image.content_url.starts_with("data:") {
                            image.content_url.to_string()
                        } else {
                            let mut image = image.clone();
                            let format = if image.content_url.ends_with(".gif") {
                                // GIF images are not supported, so convert those to PNG
                                Some(Format::Png)
                            } else {
                                None
                            };
                            embed_image(&mut image, None, format)?;
                            image.content_url
                        };

                        let pos = content_url.find(";base64,").unwrap_or(5);
                        let mime_type = &content_url[5..pos];
                        let base64 = &content_url[(pos + 8)..];
                        parts.push(Part::inline_data(mime_type, base64));
                    }
                    MessagePart::File(file) => {
                        let (Some(mime_type), Some(base64)) = (&file.media_type, &file.content)
                        else {
                            bail!("File does not have MIME type and/or content")
                        };
                        parts.push(Part::inline_data(mime_type, base64));
                    }
                    _ => {
                        tracing::warn!(
                            "User message part `{part}` is ignored by model `{}`",
                            self.id()
                        );
                    }
                }
            }

            contents.push(Content { role, parts });
        }

        let response_mime_type = match task.format {
            Some(Format::Json) => Some("application/json".to_string()),
            _ => None, // defaults to text/plain
        };

        let response_json_schema = task
            .schema
            .as_ref()
            .map(|schema| schema.clone().for_google());

        let request = GenerateContentRequest {
            contents,
            system_instruction,
            generation_config: Some(GenerationConfig {
                response_mime_type,
                response_json_schema,
                max_output_tokens: task.max_tokens,
                temperature: task.temperature,
                top_p: task.top_p,
                top_k: task.top_k,
                ..Default::default()
            }),
        };

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        let response = self
            .client
            .post(format!("{BASE_URL}/models/{0}:generateContent", self.model))
            .header("x-goog-api-key", stencila_secrets::env_or_get(API_KEY)?)
            .json(&request)
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            let message = response.text().await?;
            bail!("{error}: {message}");
        }

        let mut response: GenerateContentResponse = response.json().await?;

        let content = response
            .candidates
            .swap_remove(0)
            .content
            .parts
            .swap_remove(0);

        match content {
            Part {
                text: Some(text), ..
            } => ModelOutput::from_text(self, &task.format, text).await,
            Part {
                inline_data: Some(Blob { mime_type, data }),
                ..
            } => {
                let format = Format::from_media_type(&mime_type).ok();
                ModelOutput::from_url(self, &format, format!("{mime_type};base64,{data}")).await
            }
            _ => bail!("Unexpected response content part"),
        }
    }
}

/// A model list response
///
/// Based on https://ai.google.dev/api/rest/v1/models/list.
#[derive(Clone, Serialize, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelSpec>,
}

/// A model returned within a `ModelsResponse`
///
/// Based on https://ai.google.dev/api/rest/v1/models#Model.
/// Note: at present several fields are ignored.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelSpec {
    name: String,
    input_token_limit: Option<usize>,
}

/// A generate content request
///
/// Based on https://ai.google.dev/api/rest/v1beta/models/generateContent.
/// Note: at present several fields are ignored.
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest {
    contents: Vec<Content>,
    system_instruction: Option<Content>,
    generation_config: Option<GenerationConfig>,
}

/// A generate content response
///
/// Based on https://ai.google.dev/api/generate-content#v1beta.GenerateContentResponse.
/// Note: at present the `promptFeedback` and other fields are ignored.
#[derive(Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

/// A candidate in a generate content response
///
/// Based on https://ai.google.dev/api/generate-content#v1beta.Candidate.
/// Note: at present several fields are ignored.
#[skip_serializing_none]
#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

/// The content of a message
///
/// Based on https://ai.google.dev/api/caching#Content.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct Content {
    role: Option<Role>,
    parts: Vec<Part>,
}

/// A part of some content
///
/// Based on https://ai.google.dev/api/caching#Part.
/// Note: at present does not include all variants
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Part {
    text: Option<String>,
    inline_data: Option<Blob>,
}

impl Part {
    /// Create a new text part
    fn text(value: &str) -> Self {
        Self {
            text: Some(value.into()),
            inline_data: None,
        }
    }

    /// Create a new image data part
    #[allow(unused)]
    fn inline_data(mime_type: &str, data: &str) -> Self {
        Self {
            inline_data: Some(Blob {
                mime_type: mime_type.into(),
                data: data.into(),
            }),
            text: None,
        }
    }
}

/// Media content
///
/// Based on https://ai.google.dev/api/rest/v1beta/Content#Blob.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Blob {
    mime_type: String,
    data: String,
}

/// A role in a `Content` object
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Role {
    User,
    Model,
}

/// A configuration for generation requests
///
/// Based on https://ai.google.dev/api/generate-content#v1beta.GenerationConfig
#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    response_mime_type: Option<String>,
    response_json_schema: Option<JsonSchema>,
    stop_sequences: Option<Vec<String>>,
    candidate_count: Option<u8>,
    max_output_tokens: Option<u16>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
}

/// Get a list of available Google AI assistants
///
/// Returns an empty list if the Google AI API key is not available.
///
/// Memoized for two minutes to avoid getting secrets too frequently
/// but allowing user to set API key while process is running.
///
/// See https://ai.google.dev/api/rest/v1/models/list and
/// https://ai.google.dev/tutorials/rest_quickstart#list_models
#[cached(time = 60, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    if stencila_secrets::env_or_get(API_KEY).is_err() {
        tracing::trace!("The environment variable or secret `{API_KEY}` is not available");
        return Ok(vec![]);
    };

    let models = list_google_models()
        .await?
        .models
        .into_iter()
        .filter(|model| {
            (model.name.starts_with("models/gemini") || model.name.starts_with("models/gemma"))
                && !(model.name.contains("-embedding") || model.name.contains("-latest"))
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .map(|model| {
            let name = model.name.strip_prefix("models/").unwrap_or(&model.name);
            let context_length = model.input_token_limit.unwrap_or(4_096);

            Arc::new(GoogleModel::new(name, context_length)) as Arc<dyn Model>
        })
        .collect();

    Ok(models)
}

/// Fetch the list of models
///
/// In-memory cached for 3 hours to reduce requests to remote API.
#[cached(time = 10_800, result = true)]
async fn list_google_models() -> Result<ModelsResponse> {
    let response = Client::new()
        .get(format!("{BASE_URL}/models"))
        .header("x-goog-api-key", stencila_secrets::env_or_get(API_KEY)?)
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

        let model = GoogleModel::new("gemini-2.0-flash-001", 0);
        let output = model.perform_task(&test_task_repeat_word()).await?;

        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }
}
