use std::sync::Arc;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestMessageContentPart, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestMessageContentPartText, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest, CreateImageRequestArgs, Image, ImageDetail, ImageQuality,
        ImageSize, ImageStyle, ImageUrl, ResponseFormat, Stop,
    },
    Client,
};
use cached::proc_macro::cached;

use model::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        inflector::Inflector,
        itertools::Itertools,
        tracing,
    },
    schema::{ImageObject, MessagePart, MessageRole},
    secrets, Model, ModelIO, ModelOutput, ModelTask, ModelTaskKind, ModelType,
};

/// The name of the env var or secret for the API key
const API_KEY: &str = "OPENAI_API_KEY";

/// A model running on OpenAI
pub struct OpenAIModel {
    /// The OpenAI name for a model including any tag e.g. "llama2:13b"
    ///
    /// Used as the required `model` parameter in each request to `POST /api/generate`
    /// (along with `prompt`).
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The type of input that the model consumes
    inputs: Vec<ModelIO>,

    /// The type of output that the model generates
    outputs: Vec<ModelIO>,
}

impl OpenAIModel {
    /// Create an OpenAI-based model
    fn new(
        model: String,
        context_length: usize,
        inputs: Vec<ModelIO>,
        outputs: Vec<ModelIO>,
    ) -> Self {
        Self {
            model,
            context_length,
            inputs,
            outputs,
        }
    }
}

#[async_trait]
impl Model for OpenAIModel {
    fn name(&self) -> String {
        format!("openai/{}", self.model)
    }

    fn r#type(&self) -> ModelType {
        ModelType::Remote
    }

    fn publisher(&self) -> String {
        "OpenAI".to_string()
    }

    fn title(&self) -> String {
        if self.model.starts_with("gpt") {
            "GPT".to_string()
        } else if self.model.starts_with("tts") {
            "TTS".to_string()
        } else if self.model.starts_with("dall-e") {
            "DALL·E".to_string()
        } else {
            let name = self
                .model
                .split_once('-')
                .map(|(name, ..)| name)
                .unwrap_or(self.model.as_str());
            name.to_title_case()
        }
    }

    fn version(&self) -> String {
        let model = if self.model.starts_with("dall-e") {
            self.model.replace("dall-e", "dall_e")
        } else {
            self.model.clone()
        };
        let version = model
            .split_once('-')
            .map(|(.., version)| version)
            .unwrap_or_default();
        version.to_string()
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        &self.inputs
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        &self.outputs
    }

    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        match task.kind {
            ModelTaskKind::MessageGeneration => self.message_generation(task).await,
            ModelTaskKind::ImageGeneration => self.image_generation(task).await,
        }
    }
}

impl OpenAIModel {
    /// Create a client with the correct API key
    fn client() -> Result<Client<OpenAIConfig>> {
        let api_key = secrets::env_or_get(API_KEY)?;
        Ok(Client::with_config(
            OpenAIConfig::new().with_api_key(api_key),
        ))
    }

    #[tracing::instrument(skip_all)]
    async fn message_generation(&self, task: &ModelTask) -> Result<ModelOutput> {
        tracing::debug!("Sending chat completion request");

        let messages = task
            .messages
            .iter()
            .map(|message| match message.role.clone().unwrap_or_default() {
                MessageRole::System => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: message
                            .parts
                            .iter()
                            .filter_map(|part| match part {
                                MessagePart::Text(text) => Some(text.to_value_string()),
                                _ => {
                                    tracing::warn!(
                                        "System message part `{part}` is ignored by model `{}`",
                                        self.name()
                                    );
                                    None
                                }
                            })
                            .join("\n\n"),
                        ..Default::default()
                    })
                }
                MessageRole::User => {
                    let content = message
                        .parts
                        .iter()
                        .filter_map(|part| match part {
                            MessagePart::Text(text) => {
                                Some(ChatCompletionRequestMessageContentPart::Text(
                                    ChatCompletionRequestMessageContentPartText {
                                        text: text.to_value_string(),
                                    },
                                ))
                            }
                            MessagePart::ImageObject(ImageObject { content_url, .. }) => {
                                Some(ChatCompletionRequestMessageContentPart::ImageUrl(
                                    ChatCompletionRequestMessageContentPartImage {
                                        image_url: ImageUrl {
                                            url: content_url.clone(),
                                            detail: Some(ImageDetail::Auto),
                                        },
                                    },
                                ))
                            }
                            _ => {
                                tracing::warn!(
                                    "User message part `{part}` is ignored by model `{}`",
                                    self.name()
                                );
                                None
                            }
                        })
                        .collect_vec();

                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Array(content),
                        ..Default::default()
                    })
                }
                MessageRole::Assistant => {
                    let content = message
                        .parts
                        .iter()
                        .filter_map(|part| match part {
                            MessagePart::Text(text) => Some(text.to_value_string()),
                            _ => {
                                tracing::warn!(
                                    "Assistant message part `{part}` is ignored by model `{}`",
                                    self.name()
                                );
                                None
                            }
                        })
                        .join("");

                    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                        content: Some(content),
                        ..Default::default()
                    })
                }
            })
            .collect();

        // Create the request
        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            presence_penalty: task.repeat_penalty,
            temperature: task.temperature,
            seed: task.seed.map(|seed| seed as i64),
            max_tokens: task.max_tokens.map(|tokens| tokens as u32),
            top_p: task.top_p,
            stop: task.stop.clone().map(Stop::String),
            ..Default::default()
        };

        // Warn about ignored task options
        macro_rules! ignore_option {
            ($name:ident) => {
                if task.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by model `{}` for chat completion",
                        stringify!($name),
                        self.name()
                    )
                }
            };
            ($($name:ident),*) => {
                $( ignore_option!($name); )*
            }
        }
        ignore_option!(
            mirostat,
            mirostat_eta,
            mirostat_tau,
            num_ctx,
            num_gqa,
            num_gpu,
            num_thread,
            repeat_last_n,
            tfs_z,
            top_k
        );

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        // Send the request
        let client = Self::client()?;
        let mut response = client.chat().create(request).await?;

        // Get the content of the first message
        let text = response
            .choices
            .pop()
            .and_then(|choice| choice.message.content)
            .unwrap_or_default();

        ModelOutput::from_text(self, &task.format, text).await
    }

    #[tracing::instrument(skip_all)]
    async fn image_generation(&self, task: &ModelTask) -> Result<ModelOutput> {
        tracing::debug!("Sending image generation request");

        // Create a prompt from the last message (assumed to be a user message)
        let prompt = task
            .messages
            .last()
            .map(|message| {
                message
                    .parts
                    .iter()
                    .flat_map(|part| match part {
                        MessagePart::Text(text) => Some(text.to_value_string()),
                        _ => {
                            tracing::warn!(
                                "Message part `{part}` is ignored by model `{}`",
                                self.name()
                            );
                            None
                        }
                    })
                    .join("")
            })
            .unwrap_or_default();

        // Create the request
        let mut request = CreateImageRequestArgs::default();
        let request = request.prompt(prompt).response_format(ResponseFormat::Url);

        if let Some((w, h)) = task.image_size {
            match (w, h) {
                (256, 256) => {
                    request.size(ImageSize::S256x256);
                }
                (512, 512) => {
                    request.size(ImageSize::S512x512);
                }
                (1024, 1024) => {
                    request.size(ImageSize::S1024x1024);
                }
                (1024, 1792) => {
                    request.size(ImageSize::S1024x1792);
                }
                (1792, 1024) => {
                    request.size(ImageSize::S1792x1024);
                }
                _ => bail!("Unsupported image size `{w}x{h}`"),
            };
        }

        if let Some(quality) = &task.image_quality {
            match quality.to_lowercase().as_str() {
                "std" | "standard" => {
                    request.quality(ImageQuality::Standard);
                }
                "hd" | "high-definition" => {
                    request.quality(ImageQuality::HD);
                }
                _ => bail!("Unsupported image quality `{quality}`"),
            };
        }

        if let Some(style) = &task.image_style {
            match style.to_lowercase().as_str() {
                "nat" | "natural" => {
                    request.style(ImageStyle::Natural);
                }
                "viv" | "vivid" => {
                    request.style(ImageStyle::Vivid);
                }
                _ => bail!("Unsupported image style `{style}`"),
            };
        }

        let request = request.build()?;

        // Warn about ignored task options
        macro_rules! ignore_option {
            ($name:ident) => {
                if task.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by model `{}` for text-to-image generation",
                        stringify!($name),
                        self.name()
                    )
                }
            };
            ($($name:ident),*) => {
                $( ignore_option!($name); )*
            }
        }
        ignore_option!(
            mirostat,
            mirostat_eta,
            mirostat_tau,
            num_ctx,
            num_gqa,
            num_gpu,
            num_thread,
            repeat_last_n,
            repeat_penalty,
            temperature,
            seed,
            stop,
            max_tokens,
            tfs_z,
            top_k,
            top_p
        );

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        // Send the request
        let client = Self::client()?;
        let mut response = client.images().create(request).await?;

        // Get the output
        if response.data.is_empty() {
            bail!("Response data is unexpectedly empty")
        }
        let image = response.data.remove(0);

        match image.as_ref() {
            Image::Url { url, .. } => {
                ModelOutput::from_url(self, "image/png", url.to_string()).await
            }
            _ => bail!("Unexpected image type"),
        }
    }
}

/// Get a list of all available OpenAI models
///
/// If the OpenAI API key is not available returns an empty list.
/// Lists the models available for the account in lexical order.
///
/// This mapping of model name to context_length and input/output types will need to be
/// updated periodically based on https://platform.openai.com/docs/models/.
///
/// Memoized for an hour to reduce the number of times that
/// remote APIs need to be called to get a list of available models.
#[cached(time = 3600, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    let Ok(client) = OpenAIModel::client() else {
        tracing::trace!("The environment variable or secret `{API_KEY}` is not available");
        return Ok(vec![]);
    };

    let models = client.models().list().await?;

    let models = models
        .data
        .into_iter()
        .sorted_by(|a, b| a.id.cmp(&b.id))
        .filter_map(|model| {
            let name = model.id;

            let context_length =
                if name.starts_with("gpt-4-1106") || name.starts_with("gpt-4-vision") {
                    128_000
                } else if name.contains("-32k") {
                    32_768
                } else if name.contains("-16k") || name == "gpt-3.5-turbo-1106" {
                    16_385
                } else if name.starts_with("gpt-4") {
                    8_192
                } else if name.starts_with("dall-e-2") {
                    // Note: This seems to be characters, not tokens?
                    1_000
                } else {
                    4_096
                };

            use ModelIO::*;
            let (inputs, outputs) = if name.starts_with("gpt-4-vision") {
                (vec![Text, Image], vec![Text])
            } else if name.starts_with("gpt-4") || name.starts_with("gpt-3.5") {
                (vec![Text], vec![Text])
            } else if name.starts_with("dall-e") {
                (vec![Text], vec![Image])
            } else if name.starts_with("tts") {
                (vec![Text], vec![Audio])
            } else if name.starts_with("whisper") {
                (vec![Audio], vec![Text])
            } else {
                // Other models are not mapped
                return None;
            };

            Some(
                Arc::new(OpenAIModel::new(name, context_length, inputs, outputs)) as Arc<dyn Model>,
            )
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

        let list = list().await?;
        let model = list
            .iter()
            .find(|model| model.title().starts_with("GPT"))
            .unwrap();
        let output = model.perform_task(&test_task_repeat_word()).await?;

        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }
}
