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
    secrets, GenerateOptions, GenerateOutput, GenerateTask, Model, ModelIO, ModelType,
};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1";

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
    fn new(model: String, context_length: usize) -> Self {
        Self {
            model,
            context_length,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Model for GoogleModel {
    fn name(&self) -> String {
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

    #[tracing::instrument(skip(self))]
    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        let contents = task
            .system_prompt()
            .iter()
            .flat_map(|prompt| {
                // There is no "system" role and successive user prompts are not
                // allowed so separate any system prompt with a fake model response.
                vec![
                    Content {
                        role: Some(Role::User),
                        parts: vec![Part::text(prompt)],
                    },
                    Content {
                        role: Some(Role::Model),
                        parts: vec![Part::text("I understand those high level instructions and will follow them.")],
                    }
                ]
                .into_iter()
            })
            .chain(task.instruction_messages().iter().map(|message| {
                let role = match message.role.clone().unwrap_or_default() {
                    MessageRole::Assistant => Some(Role::Model),
                    MessageRole::User => Some(Role::User),
                    _ => None
                };

                let parts = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::Text(text) => Some(Part::text(&text.value)),
                        MessagePart::ImageObject(ImageObject{content_url,..}) => {
                            if let Some(pos) = content_url.find(";base64,") {
                                let mime_type = &content_url[..pos];
                                let base64 = &content_url[(pos + 8)..];
                                Some(Part::inline_data(mime_type, base64))
                            } else {
                                tracing::warn!(
                                    "Image does not appear to have a DataURI so was ignored by assistant `{}`",
                                    self.name()
                                );
                                None
                            }
                        },
                        _ => {
                            tracing::warn!(
                                "User message part `{part}` is ignored by assistant `{}`",
                                self.name()
                            );
                            None
                        }
                    })
                    .collect();

                Content { role, parts }
            }))
            .collect();

        let request = GenerateContentRequest {
            contents,
            generation_config: Some(GenerationConfig {
                max_output_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: options.top_p,
                top_k: options.top_k,
                ..Default::default()
            }),
        };

        if options.dry_run {
            return GenerateOutput::empty(self);
        }

        let response = self
            .client
            .post(format!(
                "{}/models/{}:generateContent",
                BASE_URL, self.model
            ))
            .query(&[("key", secrets::env_or_get(API_KEY)?)])
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
            } => {
                GenerateOutput::from_text(self, task.format(), task.instruction(), options, text)
                    .await
            }
            Part {
                inline_data: Some(Blob { mime_type, data }),
                ..
            } => {
                GenerateOutput::from_url(self, &mime_type, format!("{};base64,{}", mime_type, data))
                    .await
            }
            _ => bail!("Unexpected response content part"),
        }
    }
}

/// A model list response
///
/// Based on https://ai.google.dev/api/rest/v1/models/list.
#[derive(Deserialize)]
#[serde(crate = "model::common::serde")]
struct ModelsResponse {
    models: Vec<ModelSpec>,
}

/// A model returned within a `ModelsResponse`
///
/// Based on https://ai.google.dev/api/rest/v1/models#Model.
/// Note: at present several fields are ignored.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", crate = "model::common::serde")]
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
#[serde(rename_all = "camelCase", crate = "model::common::serde")]
struct GenerateContentRequest {
    contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
}

/// A generate content response
///
/// Based on https://ai.google.dev/api/rest/v1beta/GenerateContentResponse.
/// Note: at present the `promptFeedback` field ignored.
#[derive(Deserialize)]
#[serde(crate = "model::common::serde")]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

/// A candidate in a generate content response
///
/// Based on https://ai.google.dev/api/rest/v1beta/Candidate.
/// Note: at present several fields are ignored.
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(crate = "model::common::serde")]
struct Candidate {
    content: Content,
}

/// The content of a message
///
/// Based on https://ai.google.dev/api/rest/v1beta/Content.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(crate = "model::common::serde")]
struct Content {
    role: Option<Role>,
    parts: Vec<Part>,
}

/// A part of some content
///
/// Based on https://ai.google.dev/api/rest/v1beta/Content#Part.
/// Note: at present does not include all variants
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "model::common::serde")]
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
#[serde(rename_all = "camelCase", crate = "model::common::serde")]
struct Blob {
    mime_type: String,
    data: String,
}

/// A role in a `Content` object
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "model::common::serde")]
enum Role {
    User,
    Model,
}

/// A configuration for generation requests
///
/// Based on https://ai.google.dev/api/rest/v1beta/GenerationConfig.
#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase", crate = "model::common::serde")]
struct GenerationConfig {
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
/// Memoized for an hour to reduce the number of times that the
/// remote API need to be called to get a list of available models.
///
/// See https://ai.google.dev/api/rest/v1/models/list and
/// https://ai.google.dev/tutorials/rest_quickstart#list_models
#[cached(time = 3600, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    let Ok(key) = secrets::env_or_get(API_KEY) else {
        tracing::trace!("The environment variable or secret `{API_KEY}` is not available");
        return Ok(vec![]);
    };

    let response = Client::new()
        .get(format!("{}/models", BASE_URL))
        .query(&[("key", key)])
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        bail!(error);
    }

    let ModelsResponse { models } = response.json().await?;

    let models = models
        .into_iter()
        .filter(|model| !model.name.starts_with("models/embedding-"))
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .map(|model| {
            let name = model
                .name
                .strip_prefix("models/")
                .unwrap_or(&model.name)
                .to_string();

            let context_length = model.input_token_limit.unwrap_or(4_096);

            Arc::new(GoogleModel::new(name, context_length)) as Arc<dyn Model>
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

        let model = &list().await?[0];
        let output = model
            .perform_task(&test_task_repeat_word(), &GenerateOptions::default())
            .await?;

        assert_eq!(output.content, "HELLO".to_string());

        Ok(())
    }
}
