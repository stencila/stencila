use std::env;

use async_openai::{
    types::{
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateImageRequestArgs, Image, ImageSize, ResponseFormat,
    },
    Client,
};

use agent::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
    },
    Agent, AgentIO, GenerateImageOptions, GenerateTextOptions,
};

/// An agent running on OpenAI
///
/// The environment variable OPENAI_API_KEY must be set to use these agents.
struct OpenAIAgent {
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
    fn name(&self) -> String {
        format!("openai/{}", self.model)
    }

    fn supported_inputs(&self) -> &[AgentIO] {
        &self.inputs
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &self.outputs
    }

    async fn text_to_text(
        &self,
        instruction: &str,
        _options: Option<GenerateTextOptions>,
    ) -> Result<String> {
        let client = Client::new();

        // TODO: Add system prompt message first

        let messages = [ChatCompletionRequestUserMessageArgs::default()
            .content(instruction)
            .build()?
            .into()];

        // TODO: map options into request

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model(&self.model)
            .messages(messages)
            .build()?;

        let mut response = client.chat().create(request).await?;

        let text = response
            .choices
            .pop()
            .and_then(|choice| choice.message.content)
            .unwrap_or_default();

        Ok(text)
    }

    async fn text_to_image(
        &self,
        instruction: &str,
        _options: Option<GenerateImageOptions>,
    ) -> Result<String> {
        let client = Client::new();

        let request = CreateImageRequestArgs::default()
            .prompt(instruction)
            .response_format(ResponseFormat::Url)
            .size(ImageSize::S256x256)
            .build()?;

        let response = client.images().create(request).await?;

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

/// Get a list of all available OpenAI agents
///
/// Errors if the `OPENAI_API_KEY` env var is not set.
/// Lists the agents available to the account in descending order
/// or creation date so that more recent (i.e. "better") models are
/// first.
pub async fn list() -> Result<Vec<Box<dyn Agent>>> {
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
            } else if name.starts_with("gpt-4") {
                (vec![Text], vec![Text])
            } else if name.starts_with("gpt-3.5") {
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
            Some(Box::new(agent) as Box<dyn Agent>)
        })
        .collect();

    Ok(agents)
}
