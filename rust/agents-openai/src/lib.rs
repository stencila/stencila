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
    input: AgentIO,

    /// The type of output that the model generates
    output: AgentIO,
}

impl OpenAIAgent {
    /// Create a OpenAI-based agent
    pub fn new(model: &str, input: AgentIO, output: AgentIO) -> Self {
        Self {
            model: model.into(),
            input,
            output,
        }
    }
}

#[async_trait]
impl Agent for OpenAIAgent {
    fn name(&self) -> String {
        format!("openai/{}", self.model)
    }

    fn supports_generating(&self, output: AgentIO) -> bool {
        self.output == output
    }

    async fn generate_text(
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

    async fn generate_image(
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
pub async fn list() -> Result<Vec<Box<dyn Agent>>> {
    if env::var("OPENAI_API_KEY").is_err() {
        bail!("The OPENAI_API_KEY environment variable is not set")
    }

    use AgentIO::*;
    let models = [
        ("gpt-3.5-turbo-1106", Text, Text),
        ("gpt-3.5-turbo-0613", Text, Text),
        ("dall-e-3", Text, Image),
    ];

    let agents = models
        .into_iter()
        .map(|(model, input, output)| {
            Box::new(OpenAIAgent::new(model, input, output)) as Box<dyn Agent>
        })
        .collect();

    Ok(agents)
}
