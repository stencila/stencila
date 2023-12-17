use std::{env, sync::Arc};

use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateImageRequestArgs, Image, ImageQuality, ImageSize, ImageStyle, ResponseFormat,
    },
    Client,
};

use agent::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
        tracing,
    },
    Agent, AgentIO, GenerateContext, GenerateDetails, GenerateOptions,
};

/// An agent running on OpenAI
///
/// The environment variable OPENAI_API_KEY must be set to use these agents.
pub struct OpenAIAgent {
    /// The OpenAI name for a model including any tag e.g. "llama2:13b"
    ///
    /// Used as the required `model` parameter in each request to `POST /api/generate`
    /// (along with `prompt`).
    model: String,

    /// The type of input that the model consumes
    #[allow(unused)]
    inputs: Vec<AgentIO>,

    /// The type of output that the model generates
    outputs: Vec<AgentIO>,
}

impl OpenAIAgent {
    /// Create a OpenAI-based agent
    pub fn new(model: String, inputs: Vec<AgentIO>, outputs: Vec<AgentIO>) -> Self {
        Self {
            model,
            inputs,
            outputs,
        }
    }
}

#[async_trait]
impl Agent for OpenAIAgent {
    fn provider(&self) -> String {
        "openai".to_string()
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn supported_inputs(&self) -> &[AgentIO] {
        &self.inputs
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &self.outputs
    }

    async fn text_to_text(
        &self,
        context: GenerateContext,
        options: &GenerateOptions,
    ) -> Result<(String, GenerateDetails)> {
        chat_completion(
            &self.name(),
            &self.model,
            context.system_prompt().unwrap_or_default(),
            &[&context.user_prompt()],
            options,
        )
        .await
    }

    async fn text_to_image(&self, instruction: &str, options: &GenerateOptions) -> Result<String> {
        let client = Client::new();

        // Create the base request
        let mut request = CreateImageRequestArgs::default();
        let request = request
            .prompt(instruction)
            .response_format(ResponseFormat::Url);

        // Map options onto the request
        macro_rules! ignore_option {
            ($name:ident) => {
                if options.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by agent `{}` for text-to-image generation",
                        stringify!($name),
                        self.name()
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

        Ok(url)
    }
}

/// Complete a chat
#[tracing::instrument]
async fn chat_completion(
    agent_name: &str,
    model_name: &str,
    system_prompt: &str,
    chat: &[&str],
    options: &GenerateOptions,
) -> Result<(String, GenerateDetails)> {
    tracing::debug!("Sending chat completion response");

    let client = Client::new();

    // Create the chat with any system prompt first and then alternating
    // between user and agent messages
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
    let request = request.model(model_name).messages(messages);

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
                    "Option `{}` is ignored by agent `{}` for text-to-text generation",
                    stringify!($name),
                    agent_name
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

    // Create details of the generation
    let details = GenerateDetails {
        agent_chain: vec![agent_name.to_string()],
        generate_options: options.clone(),
        model_fingerprint: response.system_fingerprint,
        ..Default::default()
    };

    Ok((text, details))
}

/// Get a list of all available OpenAI agents
///
/// Errors if the `OPENAI_API_KEY` env var is not set.
/// Lists the agents available to the account in descending order
/// or creation date so that more recent (i.e. "better") models are
/// first.
pub async fn list() -> Result<Vec<Arc<dyn Agent>>> {
    if env::var("OPENAI_API_KEY").is_err() {
        bail!("The OPENAI_API_KEY environment variable is not set")
    }

    let client = Client::new();

    let models = client.models().list().await?;

    let agents = models
        .data
        .into_iter()
        .sorted_by(|a, b| a.created.cmp(&b.created).reverse())
        .filter_map(|model| {
            let name = model.id;

            // This mapping of model name to input/output types will need to be
            // updated periodically based on https://platform.openai.com/docs/models/
            use AgentIO::*;
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
                // Other models, are not mapped into models
                return None;
            };

            let agent = OpenAIAgent::new(name, inputs, outputs);
            Some(Arc::new(agent) as Arc<dyn Agent>)
        })
        .collect();

    Ok(agents)
}
