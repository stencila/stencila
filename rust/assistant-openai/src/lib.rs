use std::{env, sync::Arc};

use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateImageRequestArgs, Image, ImageQuality, ImageSize, ImageStyle, ResponseFormat,
    },
    Client,
};
use cached::proc_macro::cached;

use assistant::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
        tracing,
    },
    Assistant, AssistantIO, GenerateDetails, GenerateOptions, GenerateOutput, GenerateTask,
};

/// An assistant running on OpenAI
///
/// The environment variable OPENAI_API_KEY must be set to use these assistants.
pub struct OpenAIAssistant {
    /// The OpenAI name for a model including any tag e.g. "llama2:13b"
    ///
    /// Used as the required `model` parameter in each request to `POST /api/generate`
    /// (along with `prompt`).
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The type of input that the model consumes
    inputs: Vec<AssistantIO>,

    /// The type of output that the model generates
    outputs: Vec<AssistantIO>,
}

impl OpenAIAssistant {
    /// Create an OpenAI-based assistant
    fn new(
        model: String,
        context_length: usize,
        inputs: Vec<AssistantIO>,
        outputs: Vec<AssistantIO>,
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
impl Assistant for OpenAIAssistant {
    fn id(&self) -> String {
        format!("openai/{}", self.model)
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        &self.inputs
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        &self.outputs
    }

    async fn perform_task(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(GenerateOutput, GenerateDetails)> {
        use AssistantIO::*;
        match (task.input, task.output) {
            (Text, Text) => self.chat_completion(task, options).await,
            (Text, Image) => self.create_image(task, options).await,
            _ => bail!(
                "{} to {} is not supported by assistant `{}`",
                task.input,
                task.output,
                self.id()
            ),
        }
    }
}

impl OpenAIAssistant {
    #[tracing::instrument(skip(self))]
    async fn chat_completion(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(GenerateOutput, GenerateDetails)> {
        tracing::debug!("Sending chat completion request");

        let client = Client::new();

        let system_prompt = task.system_prompt().unwrap_or_default();
        let chat = &[task.user_prompt()];

        // Create the chat with any system prompt first and then alternating
        // between user and assistant messages
        let mut messages = Vec::with_capacity(chat.len() + 1);
        if !system_prompt.is_empty() {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()?
                    .into(),
            );
        }
        for (index, content) in chat.iter().enumerate() {
            let message = if index % 2 == 0 {
                ChatCompletionRequestUserMessageArgs::default()
                    .content(*content)
                    .build()?
                    .into()
            } else {
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(*content)
                    .build()?
                    .into()
            };
            messages.push(message);
        }

        // Create the request
        let mut request = CreateChatCompletionRequestArgs::default();
        let request = request.model(&self.model).messages(messages);

        // Map options onto the request
        macro_rules! map_option {
            ($from:ident, $to:ident) => {
                if let Some(value) = &options.$from {
                    request.$to(value.clone());
                }
            };
            ($name:ident) => {
                map_option!($name, $name)
            };
        }
        macro_rules! ignore_option {
            ($name:ident) => {
                if options.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by assistant `{}` for text-to-text generation",
                        stringify!($name),
                        self.id()
                    )
                }
            };
        }
        ignore_option!(mirostat);
        ignore_option!(mirostat_eta);
        ignore_option!(mirostat_tau);
        ignore_option!(num_ctx);
        ignore_option!(num_gqa);
        ignore_option!(num_gpu);
        ignore_option!(num_thread);
        ignore_option!(repeat_last_n);
        map_option!(repeat_penalty, presence_penalty);
        map_option!(temperature);
        map_option!(seed);
        map_option!(stop);
        map_option!(max_tokens);
        ignore_option!(tfs_z);
        ignore_option!(num_predict);
        ignore_option!(top_k);
        map_option!(top_p);
        ignore_option!(image_size);
        ignore_option!(image_quality);
        ignore_option!(image_style);

        // Send the request
        let request = request.build()?;
        let mut response = client.chat().create(request).await?;

        // Get the content of the first message
        let text = response
            .choices
            .pop()
            .and_then(|choice| choice.message.content)
            .unwrap_or_default();
        let output = GenerateOutput::Text(text);

        // Create details of the generation
        let details = GenerateDetails {
            assistants: vec![self.id()],
            task,
            options: options.clone(),
            fingerprint: response.system_fingerprint,
        };

        Ok((output, details))
    }

    #[tracing::instrument(skip(self))]
    async fn create_image(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(GenerateOutput, GenerateDetails)> {
        tracing::debug!("Sending create image request");

        let client = Client::new();

        let prompt = format!(
            "{system_prompt}\n\n{user_prompt}",
            system_prompt = task.system_prompt().unwrap_or_default(),
            user_prompt = task.user_prompt()
        );

        // Create the base request
        let mut request = CreateImageRequestArgs::default();
        let request = request.prompt(prompt).response_format(ResponseFormat::Url);

        // Map options onto the request
        macro_rules! ignore_option {
            ($name:ident) => {
                if options.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by assistant `{}` for text-to-image generation",
                        stringify!($name),
                        self.id()
                    )
                }
            };
        }
        ignore_option!(mirostat);
        ignore_option!(mirostat_eta);
        ignore_option!(mirostat_tau);
        ignore_option!(num_ctx);
        ignore_option!(num_gqa);
        ignore_option!(num_gpu);
        ignore_option!(num_thread);
        ignore_option!(repeat_last_n);
        ignore_option!(repeat_penalty);
        ignore_option!(temperature);
        ignore_option!(seed);
        ignore_option!(stop);
        ignore_option!(max_tokens);
        ignore_option!(tfs_z);
        ignore_option!(num_predict);
        ignore_option!(top_k);
        ignore_option!(top_p);

        if let Some((w, h)) = options.image_size {
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

        if let Some(quality) = &options.image_quality {
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

        if let Some(style) = &options.image_style {
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

        // Send the request
        let request = request.build()?;
        let response = client.images().create(request).await?;

        // Get the image URL
        let url = response
            .data
            .first()
            .and_then(|image| match image.as_ref() {
                Image::Url { url, .. } => Some(url.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let output = GenerateOutput::Image(url);

        // Create details of the generation
        let details = GenerateDetails {
            assistants: vec![self.id()],
            task,
            options: options.clone(),
            ..Default::default()
        };

        Ok((output, details))
    }
}

/// Get a list of all available OpenAI assistants
///
/// If the `OPENAI_API_KEY` env var is not set returns an empty list.
/// Lists the assistants available for the account in lexical order.
///
/// This mapping of model name to context_length and input/output types will need to be
/// updated periodically based on https://platform.openai.com/docs/models/.
///
/// Memoized for an hour to reduce the number of times that
/// remote APIs need to be called to get a list of available models.
#[cached(time = 3600, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    if env::var("OPENAI_API_KEY").is_err() {
        tracing::debug!("The OPENAI_API_KEY environment variable is not set");
        return Ok(vec![]);
    }

    let client = Client::new();
    let models = client.models().list().await?;

    let assistants = models
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
                } else {
                    4_096
                };

            use AssistantIO::*;
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
                // Other models, are not mapped into assistants
                return None;
            };

            Some(
                Arc::new(OpenAIAssistant::new(name, context_length, inputs, outputs))
                    as Arc<dyn Assistant>,
            )
        })
        .collect();

    Ok(assistants)
}
